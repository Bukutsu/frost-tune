// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse, IpcRequest, IpcResponse};
use iced::futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const CHANNEL_BUFFER_SIZE: usize = 32;
const HANDSHAKE_TIMEOUT_SECS: u64 = 120;
const ROUND_TRIP_TIMEOUT_SECS: u64 = 15;

pub struct ElevatedTransport {
    tx: mpsc::Sender<IpcRequest>,
    pending: Arc<tokio::sync::Mutex<HashMap<u64, oneshot::Sender<HelperResponse>>>>,
    next_id: AtomicU64,
    _child: Child,
}

impl ElevatedTransport {
    pub async fn spawn() -> Result<Self> {
        let current_exe = std::env::current_exe().map_err(|e| {
            AppError::general(format!("Failed to resolve current executable path: {}", e))
        })?;

        let mut child = spawn_via_pkexec(CommandSpec {
            program: current_exe,
            args: vec!["--hid-helper".to_string()],
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::general("Failed to open helper stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::general("Failed to open helper stdout"))?;

        let (request_tx, mut request_rx) = mpsc::channel::<IpcRequest>(CHANNEL_BUFFER_SIZE);
        let pending = Arc::new(tokio::sync::Mutex::new(HashMap::<
            u64,
            oneshot::Sender<HelperResponse>,
        >::new()));
        let pending_task_ref = Arc::clone(&pending);

        // Task for writing to helper stdin
        let mut framed_stdin = FramedWrite::new(stdin, LinesCodec::new());
        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                if let Ok(line) = serde_json::to_string(&request) {
                    if let Err(e) = framed_stdin.send(line).await {
                        log::error!("Failed to send request to helper: {}", e);
                        break;
                    }
                }
            }
        });

        // Task for reading from helper stdout
        let mut framed_stdout = FramedRead::new(stdout, LinesCodec::new());
        tokio::spawn(async move {
            while let Some(result) = framed_stdout.next().await {
                match result {
                    Ok(line) => {
                        if let Ok(ipc_resp) = serde_json::from_str::<IpcResponse>(&line) {
                            let mut lock = pending_task_ref.lock().await;
                            if let Some(tx) = lock.remove(&ipc_resp.id) {
                                let _ = tx.send(ipc_resp.payload);
                            } else {
                                log::warn!(
                                    "Received response for unknown/expired ID: {}",
                                    ipc_resp.id
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading from helper stdout: {}", e);
                        break;
                    }
                }
            }
        });

        let transport = ElevatedTransport {
            tx: request_tx,
            pending,
            next_id: AtomicU64::new(1),
            _child: child,
        };

        // Version check with a long timeout for the human polkit prompt
        use crate::hardware::helper_ipc::IPC_VERSION;
        match transport
            .round_trip_with_timeout(
                &HelperRequest::Version,
                std::time::Duration::from_secs(HANDSHAKE_TIMEOUT_SECS),
            )
            .await?
        {
            HelperResponse::Version { version } => {
                if version != IPC_VERSION {
                    return Err(AppError::new(
                        ErrorKind::IpcError,
                        format!(
                            "IPC Version mismatch: UI={} helper={}. Re-install the application.",
                            IPC_VERSION, version
                        ),
                    ));
                }
            }
            _ => {
                return Err(AppError::new(
                    ErrorKind::IpcError,
                    "Elevated helper failed version handshake",
                ));
            }
        }

        Ok(transport)
    }

    pub async fn round_trip(&self, request: &HelperRequest) -> Result<HelperResponse> {
        self.round_trip_with_timeout(
            request,
            std::time::Duration::from_secs(ROUND_TRIP_TIMEOUT_SECS),
        )
        .await
    }

    pub async fn round_trip_with_timeout(
        &self,
        request: &HelperRequest,
        timeout: std::time::Duration,
    ) -> Result<HelperResponse> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut lock = self.pending.lock().await;
            lock.insert(id, tx);
        }

        let ipc_req = IpcRequest {
            id,
            payload: request.clone(),
        };

        if let Err(e) = self.tx.send(ipc_req).await {
            let mut lock = self.pending.lock().await;
            lock.remove(&id);
            return Err(AppError::new(
                ErrorKind::IpcError,
                format!("Failed to send request to actor: {}", e),
            ));
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => Err(AppError::new(
                ErrorKind::IpcError,
                "Helper response channel closed prematurely",
            )),
            Err(_) => {
                let mut lock = self.pending.lock().await;
                lock.remove(&id);
                Err(AppError::new(
                    ErrorKind::ReadTimeout,
                    format!("Elevated helper response timed out ({:?})", timeout),
                ))
            }
        }
    }

    pub fn shutdown(&mut self) {
        // In the async version, dropping the tx will close the write task.
    }
}

impl crate::hardware::transport::Transport for ElevatedTransport {
    fn round_trip<'a>(
        &'a self,
        request: &'a HelperRequest,
    ) -> iced::futures::future::BoxFuture<'a, Result<HelperResponse>> {
        Box::pin(async move { self.round_trip(request).await })
    }

    fn shutdown(&mut self) {
        self.shutdown();
    }
}

struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
}

fn spawn_via_pkexec(spec: CommandSpec) -> Result<Child> {
    validate_pkexec_target(&spec.program)?;
    let mut command = Command::new("pkexec");
    command.arg(spec.program.as_os_str());
    for arg in spec.args {
        command.arg(arg);
    }

    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::new(
                    ErrorKind::PolkitAuthRequired,
                    "pkexec not found. Install polkit (policykit-1).",
                )
            } else {
                AppError::general(format!("Failed to launch helper via pkexec: {}", e))
            }
        })?;

    Ok(child)
}

fn validate_pkexec_target(path: &std::path::Path) -> Result<()> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        AppError::new(
            ErrorKind::IpcError,
            format!("Failed to stat executable: {}", e),
        )
    })?;

    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(AppError::new(
            ErrorKind::IpcError,
            "Executable is group-writable or world-writable; refusing to elevate.",
        ));
    }

    let exe_owner = metadata.uid();
    if exe_owner != 0 {
        return Err(AppError::new(
            ErrorKind::IpcError,
            "Executable must be owned by root to prevent TOCTOU attacks when elevating.",
        ));
    }

    let parent = path.parent().ok_or_else(|| {
        AppError::new(
            ErrorKind::IpcError,
            "Failed to resolve executable directory",
        )
    })?;
    let parent_meta = std::fs::metadata(parent).map_err(|e| {
        AppError::new(
            ErrorKind::IpcError,
            format!("Failed to stat executable directory: {}", e),
        )
    })?;

    if parent_meta.permissions().mode() & 0o002 != 0 {
        return Err(AppError::new(
            ErrorKind::IpcError,
            "Executable directory is world-writable; refusing to elevate.",
        ));
    }

    let uid = nix::unistd::Uid::current().as_raw();
    let owner = parent_meta.uid();
    if owner != 0 && owner != uid {
        return Err(AppError::new(
            ErrorKind::IpcError,
            "Executable directory is not owned by current user or root.",
        ));
    }

    Ok(())
}
