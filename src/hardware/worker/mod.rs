// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::sync::mpsc as std_mpsc;
use tokio::sync::{mpsc, oneshot};

use crate::core::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
use crate::error::{AppError, ErrorKind};

pub mod backend;
pub mod local_thread;

pub use backend::BackendKind;
use local_thread::{run_local_worker, LocalCommand, LocalStatus};

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

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
    PushPEQ(PushPayload, oneshot::Sender<OperationResult>),
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

    pub async fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> Result<ConnectionResult, String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(UsbCommand::Connect(device, backend, tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker dropped connection channel".to_string())
    }

    pub async fn disconnect(&self) -> Result<OperationResult, String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(UsbCommand::Disconnect(tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker dropped disconnect channel".to_string())
    }

    pub async fn status(&self) -> Result<WorkerStatus, String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(UsbCommand::Status(tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker dropped status channel".to_string())
    }

    pub async fn pull_peq(&self) -> Result<OperationResult, String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(UsbCommand::PullPEQ(tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker dropped pull channel".to_string())
    }

    pub async fn push_peq(&self, payload: PushPayload) -> Result<OperationResult, String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(UsbCommand::PushPEQ(payload, tx))
            .await
            .map_err(|_| "Worker queue is closed".to_string())?;
        rx.await
            .map_err(|_| "Worker dropped push channel".to_string())
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
    }
}

struct UsbWorkerState {
    local_tx: std_mpsc::Sender<LocalCommand>,
    backend_kind: Option<BackendKind>,
    preferred_backend: BackendKind,

    #[cfg(target_os = "linux")]
    elevated_transport: Option<ElevatedTransport>,
    #[cfg(target_os = "linux")]
    elevated_info: Option<DeviceInfo>,
}

impl UsbWorkerState {
    fn new(local_tx: std_mpsc::Sender<LocalCommand>) -> Self {
        Self {
            local_tx,
            backend_kind: None,
            preferred_backend: BackendKind::Local,
            #[cfg(target_os = "linux")]
            elevated_transport: None,
            #[cfg(target_os = "linux")]
            elevated_info: None,
        }
    }

    async fn process_command(&mut self, cmd: UsbCommand) {
        match cmd {
            UsbCommand::Connect(target_device, target_backend, resp) => {
                let target = target_backend.unwrap_or(self.preferred_backend);

                #[cfg(target_os = "linux")]
                if target == BackendKind::Elevated {
                    if let Ok(transport) = ElevatedTransport::spawn().await {
                        if let Ok(HelperResponse::Connected { device: Some(info) }) =
                            transport.round_trip(&HelperRequest::Connect).await
                        {
                            self.elevated_transport = Some(transport);
                            self.elevated_info = Some(info.clone());
                            self.backend_kind = Some(BackendKind::Elevated);
                            self.preferred_backend = BackendKind::Elevated;

                            let (ltx, _) = oneshot::channel();
                            let _ = self.local_tx.send(LocalCommand::Disconnect(ltx));

                            let _ = resp.send(ConnectionResult {
                                success: true,
                                device: Some(info),
                                error: None,
                            });
                            return;
                        }
                    }
                }

                // Fallback to local
                let (ltx, lrx) = oneshot::channel();
                if self
                    .local_tx
                    .send(LocalCommand::Connect(target_device.clone(), ltx))
                    .is_ok()
                {
                    if let Ok(res) = lrx.await {
                        if res.success {
                            self.backend_kind = Some(BackendKind::Local);
                            self.preferred_backend = BackendKind::Local;
                            #[cfg(target_os = "linux")]
                            {
                                self.elevated_transport = None;
                                self.elevated_info = None;
                            }
                        } else {
                            #[cfg(target_os = "linux")]
                            if res
                                .error
                                .as_ref()
                                .is_some_and(|e| e.kind == ErrorKind::PermissionDenied)
                            {
                                if let Ok(transport) = ElevatedTransport::spawn().await {
                                    if let Ok(HelperResponse::Connected { device: Some(info) }) =
                                        transport.round_trip(&HelperRequest::Connect).await
                                    {
                                        self.elevated_transport = Some(transport);
                                        self.elevated_info = Some(info.clone());
                                        self.backend_kind = Some(BackendKind::Elevated);
                                        self.preferred_backend = BackendKind::Elevated;
                                        let _ = resp.send(ConnectionResult {
                                            success: true,
                                            device: Some(info),
                                            error: None,
                                        });
                                        return;
                                    }
                                }
                            }
                        }
                        let _ = resp.send(res);
                        return;
                    }
                }
                let _ = resp.send(ConnectionResult {
                    success: false,
                    device: None,
                    error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
                });
            }
            UsbCommand::Disconnect(resp) => {
                #[cfg(target_os = "linux")]
                if self.backend_kind == Some(BackendKind::Elevated) {
                    if let Some(transport) = &mut self.elevated_transport {
                        let _ = transport.round_trip(&HelperRequest::Disconnect).await;
                        transport.shutdown();
                    }
                    self.elevated_transport = None;
                    self.elevated_info = None;
                    self.backend_kind = None;
                    let _ = resp.send(OperationResult {
                        success: true,
                        data: None,
                        error: None,
                    });
                    return;
                }

                self.backend_kind = None;
                let (ltx, lrx) = oneshot::channel();
                let _ = self.local_tx.send(LocalCommand::Disconnect(ltx));
                let _ = resp.send(lrx.await.unwrap_or(OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
                }));
            }
            UsbCommand::Status(resp) => {
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

                let mut final_status = WorkerStatus {
                    connected: local_status.connected,
                    physically_present: local_status.physically_present,
                    device: local_status.device.clone(),
                    available_devices: local_status.available_devices.clone(),
                    backend_reset: local_status.backend_reset,
                    generation: local_status.generation,
                    fatal_error: local_status.fatal_error.clone(),
                };

                #[cfg(target_os = "linux")]
                if self.backend_kind == Some(BackendKind::Elevated) {
                    if let Some(transport) = &mut self.elevated_transport {
                        match transport.round_trip(&HelperRequest::Status).await {
                            Ok(HelperResponse::Status {
                                connected,
                                physically_present,
                                device,
                            }) => {
                                if !connected {
                                    self.backend_kind = None;
                                    self.elevated_transport = None;
                                }
                                final_status.connected = connected;
                                final_status.physically_present = physically_present;
                                final_status.device = device.or(self.elevated_info.clone());
                            }
                            _ => {
                                self.backend_kind = None;
                                self.elevated_transport = None;
                                final_status.connected = false;
                                final_status.backend_reset = true;
                            }
                        }
                    } else {
                        self.backend_kind = None;
                        final_status.connected = false;
                    }
                } else if self.backend_kind == Some(BackendKind::Local) && !local_status.connected {
                    self.backend_kind = None;
                }

                let _ = resp.send(final_status);
            }
            UsbCommand::PullPEQ(resp) => {
                #[cfg(target_os = "linux")]
                if self.backend_kind == Some(BackendKind::Elevated) {
                    if let Some(transport) = &mut self.elevated_transport {
                        match transport
                            .round_trip(&HelperRequest::PullPeq { strict: false })
                            .await
                        {
                            Ok(HelperResponse::Pulled { data }) => {
                                match serde_json::from_value(data) {
                                    Ok(peq) => {
                                        let _ = resp.send(OperationResult {
                                            success: true,
                                            data: Some(peq),
                                            error: None,
                                        });
                                        return;
                                    }
                                    Err(e) => {
                                        let _ = resp.send(OperationResult {
                                            success: false,
                                            data: None,
                                            error: Some(AppError::new(
                                                ErrorKind::ParseError,
                                                e.to_string(),
                                            )),
                                        });
                                        return;
                                    }
                                }
                            }
                            Ok(HelperResponse::Error { error, .. }) => {
                                let _ = resp.send(OperationResult {
                                    success: false,
                                    data: None,
                                    error: Some(error),
                                });
                                return;
                            }
                            _ => {
                                self.backend_kind = None;
                                self.elevated_transport = None;
                                let _ = resp.send(OperationResult {
                                    success: false,
                                    data: None,
                                    error: Some(AppError::new(
                                        ErrorKind::IpcError,
                                        "Elevated helper died",
                                    )),
                                });
                                return;
                            }
                        }
                    }
                }

                let (ltx, lrx) = oneshot::channel();
                let _ = self.local_tx.send(LocalCommand::PullPEQ(ltx));
                let _ = resp.send(lrx.await.unwrap_or(OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
                }));
            }
            UsbCommand::PushPEQ(payload, resp) => {
                #[cfg(target_os = "linux")]
                if self.backend_kind == Some(BackendKind::Elevated) {
                    if let Some(transport) = &mut self.elevated_transport {
                        match transport
                            .round_trip(&HelperRequest::PushPeq {
                                filters: payload.filters.clone(),
                                global_gain: payload.global_gain,
                            })
                            .await
                        {
                            Ok(HelperResponse::Pushed { data }) => {
                                match serde_json::from_value(data) {
                                    Ok(peq) => {
                                        let _ = resp.send(OperationResult {
                                            success: true,
                                            data: Some(peq),
                                            error: None,
                                        });
                                        return;
                                    }
                                    Err(e) => {
                                        let _ = resp.send(OperationResult {
                                            success: false,
                                            data: None,
                                            error: Some(AppError::new(
                                                ErrorKind::ParseError,
                                                e.to_string(),
                                            )),
                                        });
                                        return;
                                    }
                                }
                            }
                            Ok(HelperResponse::Error { error, .. }) => {
                                let _ = resp.send(OperationResult {
                                    success: false,
                                    data: None,
                                    error: Some(error),
                                });
                                return;
                            }
                            _ => {
                                self.backend_kind = None;
                                self.elevated_transport = None;
                                let _ = resp.send(OperationResult {
                                    success: false,
                                    data: None,
                                    error: Some(AppError::new(
                                        ErrorKind::IpcError,
                                        "Elevated helper died",
                                    )),
                                });
                                return;
                            }
                        }
                    }
                }

                let (ltx, lrx) = oneshot::channel();
                let _ = self.local_tx.send(LocalCommand::PushPEQ(payload, ltx));
                let _ = resp.send(lrx.await.unwrap_or(OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
                }));
            }
        }
    }
}
