// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::sync::mpsc as std_mpsc;
use tokio::sync::oneshot;

use crate::core::DeviceInfo;
use crate::error::{AppError, ErrorKind};
use crate::hardware::ConnectionResult;

use super::{ActiveBackend, BackendKind, LocalCommand};

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};
pub struct ConnectionManager {
    pub(crate) backend: Option<ActiveBackend>,
    pub preferred_backend: BackendKind,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            backend: None,
            preferred_backend: BackendKind::Local,
        }
    }

    pub async fn handle_connect(
        &mut self,
        local_tx: &std_mpsc::Sender<LocalCommand>,
        target_device: Option<DeviceInfo>,
        target_backend: Option<BackendKind>,
    ) -> ConnectionResult {
        #[cfg(target_os = "linux")]
        let target = target_backend.unwrap_or(self.preferred_backend);
        #[cfg(not(target_os = "linux"))]
        let _target = target_backend.unwrap_or(self.preferred_backend);

        #[cfg(target_os = "linux")]
        if target == BackendKind::Elevated {
            return self.connect_elevated(local_tx, target_device).await;
        }

        self.connect_local(local_tx, target_device).await
    }

    async fn connect_local(
        &mut self,
        local_tx: &std_mpsc::Sender<LocalCommand>,
        target_device: Option<DeviceInfo>,
    ) -> ConnectionResult {
        let (ltx, lrx) = oneshot::channel();
        if local_tx
            .send(LocalCommand::Connect(target_device.clone(), ltx))
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
                    return self.connect_elevated(local_tx, target_device).await;
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
    async fn connect_elevated(
        &mut self,
        local_tx: &std_mpsc::Sender<LocalCommand>,
        target_device: Option<DeviceInfo>,
    ) -> ConnectionResult {
        match ElevatedTransport::spawn().await {
            Ok(transport) => {
                match transport
                    .round_trip(&HelperRequest::Connect {
                        device: target_device,
                    })
                    .await
                {
                    Ok(HelperResponse::Connected { device: Some(info) }) => {
                        let (ltx, _) = oneshot::channel();
                        let _ = local_tx.send(LocalCommand::Disconnect(ltx));
                        self.backend = Some(ActiveBackend::Elevated {
                            transport: Box::new(transport),
                            device_info: Some(info.clone()),
                        });
                        self.preferred_backend = BackendKind::Elevated;
                        ConnectionResult {
                            success: true,
                            device: Some(info),
                            error: None,
                        }
                    }
                    Ok(HelperResponse::Error { error }) => ConnectionResult {
                        success: false,
                        device: None,
                        error: Some(error),
                    },
                    Ok(_) => ConnectionResult {
                        success: false,
                        device: None,
                        error: Some(AppError::new(
                            ErrorKind::IpcError,
                            "Elevated helper handshake failed",
                        )),
                    },
                    Err(e) => ConnectionResult {
                        success: false,
                        device: None,
                        error: Some(e),
                    },
                }
            }
            Err(e) => ConnectionResult {
                success: false,
                device: None,
                error: Some(e),
            },
        }
    }

    pub async fn handle_disconnect(
        &mut self,
        local_tx: &std_mpsc::Sender<LocalCommand>,
    ) -> crate::hardware::OperationResult {
        let mut result = crate::hardware::OperationResult {
            success: true,
            data: None,
            error: None,
        };

        if matches!(self.backend, Some(ActiveBackend::Local)) {
            let (ltx, lrx) = oneshot::channel();
            if local_tx.send(LocalCommand::Disconnect(ltx)).is_ok() {
                if let Ok(res) = lrx.await {
                    result = res;
                }
            }
        }

        self.backend = None;
        result
    }
}
