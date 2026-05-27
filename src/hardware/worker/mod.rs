// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::sync::mpsc as std_mpsc;
use tokio::sync::{mpsc, oneshot};

use crate::core::DeviceInfo;
use crate::error::{AppError, ErrorKind};
use crate::hardware::{ConnectionResult, OperationResult, PushPayload};

pub mod backend;
pub mod connection;
pub mod discovery;
pub mod local_thread;

pub use backend::BackendKind;
use local_thread::{run_local_worker, LocalCommand, LocalStatus};

#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};
#[cfg(target_os = "linux")]
use crate::hardware::transport::Transport;
#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub connected: bool,
    pub physically_present: bool,
    pub device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
    pub backend_reset: bool,
    pub generation: u64,
    pub fatal_error: Option<String>,
}

pub enum UsbCommand {
    Connect(
        Option<DeviceInfo>,
        Option<BackendKind>,
        oneshot::Sender<ConnectionResult>,
    ),
    Disconnect(oneshot::Sender<OperationResult>),
    Status(oneshot::Sender<WorkerStatus>),
    PullPEQ(oneshot::Sender<OperationResult>),
    PushPEQ(PushPayload, bool, oneshot::Sender<OperationResult>),
    ResetPEQ(oneshot::Sender<OperationResult>),
}

pub struct UsbWorker {
    tx: mpsc::Sender<UsbCommand>,
}

impl UsbWorker {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(32);

        let (local_tx, local_rx) = std_mpsc::channel();
        std::thread::spawn(move || run_local_worker(local_rx));

        tokio::spawn(async move {
            let mut state = UsbWorkerState::new(local_tx);
            while let Some(cmd) = rx.recv().await {
                state.process_command(cmd).await;
            }
        });

        UsbWorker { tx }
    }

    async fn send_command<F, R>(&self, build: F) -> Result<R, String>
    where
        F: FnOnce(oneshot::Sender<R>) -> UsbCommand,
    {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(build(tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker response channel closed".to_string())
    }

    pub async fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> Result<ConnectionResult, String> {
        self.send_command(|tx| UsbCommand::Connect(device, backend, tx))
            .await
    }

    pub async fn disconnect(&self) -> Result<OperationResult, String> {
        self.send_command(UsbCommand::Disconnect).await
    }

    pub async fn status(&self) -> Result<WorkerStatus, String> {
        self.send_command(UsbCommand::Status).await
    }

    pub async fn pull_peq(&self) -> Result<OperationResult, String> {
        self.send_command(UsbCommand::PullPEQ).await
    }

    pub async fn push_peq(
        &self,
        payload: PushPayload,
        skip_verify: bool,
    ) -> Result<OperationResult, String> {
        self.send_command(|tx| UsbCommand::PushPEQ(payload, skip_verify, tx))
            .await
    }

    pub async fn reset_peq(&self) -> Result<OperationResult, String> {
        self.send_command(UsbCommand::ResetPEQ).await
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Active backend ───────────────────────────────────────────────────────────

pub(crate) enum ActiveBackend {
    Local,
    #[cfg(target_os = "linux")]
    Elevated {
        transport: Box<dyn Transport>,
        device_info: Option<DeviceInfo>,
    },
}

// ─── Worker state ─────────────────────────────────────────────────────────────

struct UsbWorkerState {
    local_tx: std_mpsc::Sender<LocalCommand>,
    connection_manager: connection::ConnectionManager,
}

impl UsbWorkerState {
    fn new(local_tx: std_mpsc::Sender<LocalCommand>) -> Self {
        Self {
            local_tx,
            connection_manager: connection::ConnectionManager::new(),
        }
    }

    async fn process_command(&mut self, cmd: UsbCommand) {
        match cmd {
            UsbCommand::Connect(device, backend, resp) => {
                let _ = resp.send(
                    self.connection_manager
                        .handle_connect(&self.local_tx, device, backend)
                        .await,
                );
            }
            UsbCommand::Disconnect(resp) => {
                let _ = resp.send(
                    self.connection_manager
                        .handle_disconnect(&self.local_tx)
                        .await,
                );
            }
            UsbCommand::Status(resp) => {
                let _ = resp.send(self.handle_status().await);
            }
            UsbCommand::PullPEQ(resp) => {
                let _ = resp.send(self.handle_pull().await);
            }
            UsbCommand::PushPEQ(payload, skip_verify, resp) => {
                let _ = resp.send(self.handle_push(payload, skip_verify).await);
            }
            UsbCommand::ResetPEQ(resp) => {
                let _ = resp.send(self.handle_reset().await);
            }
        }
    }

    async fn handle_status(&mut self) -> WorkerStatus {
        let (ltx, lrx) = oneshot::channel();
        let _ = self.local_tx.send(LocalCommand::Status(ltx));
        let local_status = lrx.await.unwrap_or(LocalStatus {
            connected: false,
            physically_present: false,
            device: None,
            available_devices: vec![],
            backend_reset: false,
            generation: 0,
            fatal_error: Some("Local worker thread died".to_string()),
        });

        let mut status = WorkerStatus {
            connected: local_status.connected,
            physically_present: local_status.physically_present,
            device: local_status.device.clone(),
            available_devices: local_status.available_devices.clone(),
            backend_reset: local_status.backend_reset,
            generation: local_status.generation,
            fatal_error: local_status.fatal_error.clone(),
        };

        #[cfg(target_os = "linux")]
        {
            // Immutable borrow ends before any self.connection_manager.backend mutation below.
            let elevated_result = if let Some(ActiveBackend::Elevated { transport, .. }) =
                &self.connection_manager.backend
            {
                Some(transport.round_trip(&HelperRequest::Status).await)
            } else {
                None
            };

            match elevated_result {
                Some(Ok(HelperResponse::Status {
                    connected,
                    physically_present,
                    device,
                })) => {
                    let fallback = if let Some(ActiveBackend::Elevated { device_info, .. }) =
                        &self.connection_manager.backend
                    {
                        device_info.clone()
                    } else {
                        None
                    };
                    status.connected = connected;
                    status.physically_present = physically_present;
                    status.device = device.or(fallback);
                    if !connected {
                        self.connection_manager.backend = None;
                    }
                }
                Some(_) => {
                    self.connection_manager.backend = None;
                    status.connected = false;
                    status.backend_reset = true;
                }
                None => {
                    if matches!(self.connection_manager.backend, Some(ActiveBackend::Local))
                        && !local_status.connected
                    {
                        self.connection_manager.backend = None;
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        if matches!(self.connection_manager.backend, Some(ActiveBackend::Local))
            && !local_status.connected
        {
            self.connection_manager.backend = None;
        }

        status
    }

    async fn handle_pull(&mut self) -> OperationResult {
        #[cfg(target_os = "linux")]
        {
            let elevated_result = if let Some(ActiveBackend::Elevated { transport, .. }) =
                &self.connection_manager.backend
            {
                Some(
                    transport
                        .round_trip(&HelperRequest::PullPeq { strict: false })
                        .await,
                )
            } else {
                None
            };

            if let Some(response) = elevated_result {
                return match response {
                    Ok(HelperResponse::Pulled { data }) => match serde_json::from_value(data) {
                        Ok(peq) => OperationResult {
                            success: true,
                            data: Some(peq),
                            error: None,
                        },
                        Err(e) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::ParseError, e.to_string())),
                        },
                    },
                    Ok(HelperResponse::Error { error, .. }) => OperationResult {
                        success: false,
                        data: None,
                        error: Some(error),
                    },
                    _ => {
                        self.connection_manager.backend = None;
                        OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::IpcError, "Elevated helper died")),
                        }
                    }
                };
            }
        }

        let (ltx, lrx) = oneshot::channel();
        let _ = self.local_tx.send(LocalCommand::PullPEQ(ltx));
        lrx.await.unwrap_or(OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        })
    }

    async fn handle_push(&mut self, payload: PushPayload, skip_verify: bool) -> OperationResult {
        #[cfg(target_os = "linux")]
        {
            let elevated_result = if let Some(ActiveBackend::Elevated { transport, .. }) =
                &self.connection_manager.backend
            {
                Some(
                    transport
                        .round_trip(&HelperRequest::PushPeq {
                            filters: payload.filters.clone(),
                            global_gain: payload.global_gain,
                            skip_verify,
                        })
                        .await,
                )
            } else {
                None
            };

            if let Some(response) = elevated_result {
                return match response {
                    Ok(HelperResponse::Pushed { data }) => match serde_json::from_value(data) {
                        Ok(peq) => OperationResult {
                            success: true,
                            data: Some(peq),
                            error: None,
                        },
                        Err(e) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::ParseError, e.to_string())),
                        },
                    },
                    Ok(HelperResponse::Error { error, .. }) => OperationResult {
                        success: false,
                        data: None,
                        error: Some(error),
                    },
                    _ => {
                        self.connection_manager.backend = None;
                        OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::IpcError, "Elevated helper died")),
                        }
                    }
                };
            }
        }

        let (ltx, lrx) = oneshot::channel();
        let _ = self
            .local_tx
            .send(LocalCommand::PushPEQ(payload, skip_verify, ltx));
        lrx.await.unwrap_or(OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        })
    }

    async fn handle_reset(&mut self) -> OperationResult {
        #[cfg(target_os = "linux")]
        {
            let elevated_result = if let Some(ActiveBackend::Elevated { transport, .. }) =
                &self.connection_manager.backend
            {
                Some(transport.round_trip(&HelperRequest::ResetPeq).await)
            } else {
                None
            };

            if let Some(response) = elevated_result {
                return match response {
                    Ok(HelperResponse::Pulled { data }) => match serde_json::from_value(data) {
                        Ok(peq) => OperationResult {
                            success: true,
                            data: Some(peq),
                            error: None,
                        },
                        Err(e) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::ParseError, e.to_string())),
                        },
                    },
                    Ok(HelperResponse::Error { error, .. }) => OperationResult {
                        success: false,
                        data: None,
                        error: Some(error),
                    },
                    _ => {
                        self.connection_manager.backend = None;
                        OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::IpcError, "Elevated helper died")),
                        }
                    }
                };
            }
        }

        let (ltx, lrx) = oneshot::channel();
        let _ = self.local_tx.send(LocalCommand::ResetPEQ(ltx));
        lrx.await.unwrap_or(OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        })
    }
}
