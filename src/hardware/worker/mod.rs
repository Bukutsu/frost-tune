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

// ─── Active backend ───────────────────────────────────────────────────────────

enum ActiveBackend {
    Local,
    #[cfg(target_os = "linux")]
    Elevated {
        transport: Box<ElevatedTransport>,
        device_info: Option<DeviceInfo>,
    },
}

// ─── Worker state ─────────────────────────────────────────────────────────────

struct UsbWorkerState {
    local_tx: std_mpsc::Sender<LocalCommand>,
    backend: Option<ActiveBackend>,
    preferred_backend: BackendKind,
}

impl UsbWorkerState {
    fn new(local_tx: std_mpsc::Sender<LocalCommand>) -> Self {
        Self {
            local_tx,
            backend: None,
            preferred_backend: BackendKind::Local,
        }
    }

    async fn process_command(&mut self, cmd: UsbCommand) {
        match cmd {
            UsbCommand::Connect(device, backend, resp) => {
                let _ = resp.send(self.handle_connect(device, backend).await);
            }
            UsbCommand::Disconnect(resp) => {
                let _ = resp.send(self.handle_disconnect().await);
            }
            UsbCommand::Status(resp) => {
                let _ = resp.send(self.handle_status().await);
            }
            UsbCommand::PullPEQ(resp) => {
                let _ = resp.send(self.handle_pull().await);
            }
            UsbCommand::PushPEQ(payload, resp) => {
                let _ = resp.send(self.handle_push(payload).await);
            }
        }
    }

    async fn handle_connect(
        &mut self,
        target_device: Option<DeviceInfo>,
        target_backend: Option<BackendKind>,
    ) -> ConnectionResult {
        let target = target_backend.unwrap_or(self.preferred_backend);

        #[cfg(target_os = "linux")]
        if target == BackendKind::Elevated {
            return self.connect_elevated().await;
        }

        self.connect_local(target_device).await
    }

    async fn connect_local(&mut self, target_device: Option<DeviceInfo>) -> ConnectionResult {
        let (ltx, lrx) = oneshot::channel();
        if self
            .local_tx
            .send(LocalCommand::Connect(target_device, ltx))
            .is_ok()
        {
            if let Ok(res) = lrx.await {
                if res.success {
                    self.backend = Some(ActiveBackend::Local);
                    self.preferred_backend = BackendKind::Local;
                    return res;
                }

                #[cfg(target_os = "linux")]
                if res
                    .error
                    .as_ref()
                    .is_some_and(|e| e.kind == ErrorKind::PermissionDenied)
                {
                    let elevated = self.connect_elevated().await;
                    if elevated.success {
                        return elevated;
                    }
                }

                return res;
            }
        }
        ConnectionResult {
            success: false,
            device: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        }
    }

    #[cfg(target_os = "linux")]
    async fn connect_elevated(&mut self) -> ConnectionResult {
        if let Ok(transport) = ElevatedTransport::spawn().await {
            if let Ok(HelperResponse::Connected { device: Some(info) }) =
                transport.round_trip(&HelperRequest::Connect).await
            {
                let (ltx, _) = oneshot::channel();
                let _ = self.local_tx.send(LocalCommand::Disconnect(ltx));
                self.backend = Some(ActiveBackend::Elevated {
                    transport: Box::new(transport),
                    device_info: Some(info.clone()),
                });
                self.preferred_backend = BackendKind::Elevated;
                return ConnectionResult {
                    success: true,
                    device: Some(info),
                    error: None,
                };
            }
        }
        ConnectionResult {
            success: false,
            device: None,
            error: Some(AppError::new(
                ErrorKind::PermissionDenied,
                "Elevated transport failed to connect",
            )),
        }
    }

    async fn handle_disconnect(&mut self) -> OperationResult {
        #[cfg(target_os = "linux")]
        if let Some(ActiveBackend::Elevated { .. }) = &self.backend {
            if let Some(ActiveBackend::Elevated {
                transport: mut t, ..
            }) = self.backend.take()
            {
                let _ = t.round_trip(&HelperRequest::Disconnect).await;
                t.shutdown();
            }
            return OperationResult {
                success: true,
                data: None,
                error: None,
            };
        }

        self.backend = None;
        let (ltx, lrx) = oneshot::channel();
        let _ = self.local_tx.send(LocalCommand::Disconnect(ltx));
        lrx.await.unwrap_or(OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        })
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
            // Immutable borrow ends before any self.backend mutation below.
            let elevated_result =
                if let Some(ActiveBackend::Elevated { transport, .. }) = &self.backend {
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
                    let fallback =
                        if let Some(ActiveBackend::Elevated { device_info, .. }) = &self.backend {
                            device_info.clone()
                        } else {
                            None
                        };
                    status.connected = connected;
                    status.physically_present = physically_present;
                    status.device = device.or(fallback);
                    if !connected {
                        self.backend = None;
                    }
                }
                Some(_) => {
                    self.backend = None;
                    status.connected = false;
                    status.backend_reset = true;
                }
                None => {
                    if matches!(self.backend, Some(ActiveBackend::Local)) && !local_status.connected
                    {
                        self.backend = None;
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        if matches!(self.backend, Some(ActiveBackend::Local)) && !local_status.connected {
            self.backend = None;
        }

        status
    }

    async fn handle_pull(&mut self) -> OperationResult {
        #[cfg(target_os = "linux")]
        {
            let elevated_result =
                if let Some(ActiveBackend::Elevated { transport, .. }) = &self.backend {
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
                        self.backend = None;
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

    async fn handle_push(&mut self, payload: PushPayload) -> OperationResult {
        #[cfg(target_os = "linux")]
        {
            let elevated_result =
                if let Some(ActiveBackend::Elevated { transport, .. }) = &self.backend {
                    Some(
                        transport
                            .round_trip(&HelperRequest::PushPeq {
                                filters: payload.filters.clone(),
                                global_gain: payload.global_gain,
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
                        self.backend = None;
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
        let _ = self.local_tx.send(LocalCommand::PushPEQ(payload, ltx));
        lrx.await.unwrap_or(OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::Unknown, "Worker thread died")),
        })
    }
}
