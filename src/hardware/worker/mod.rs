pub mod backend;
pub mod connection;
pub mod ops;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::hardware::hid::find_device_info;
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
pub use backend::BackendKind;
use crate::hardware::worker::backend::TransportBackend;
use crate::hardware::worker::connection::worker_connect;
use crate::hardware::worker::ops::{worker_pull_peq, worker_push_peq};

#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub connected: bool,
    pub physically_present: bool,
    pub device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
}

pub enum UsbCommand {
    Connect(Option<DeviceInfo>, Option<BackendKind>, mpsc::Sender<ConnectionResult>),
    Disconnect(mpsc::Sender<OperationResult>),
    Status(mpsc::Sender<WorkerStatus>),
    PullPEQ(mpsc::Sender<OperationResult>),
    PushPEQ(PushPayload, mpsc::Sender<OperationResult>),
}

pub struct UsbWorker {
    tx: mpsc::Sender<UsbCommand>,
}

impl UsbWorker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut backend: Option<TransportBackend> = None;
            let mut preferred_backend: BackendKind = BackendKind::Local;

            let mut api = match hidapi::HidApi::new() {
                Ok(a) => {
                    log::info!("HID API initialized successfully");
                    a
                }
                Err(e) => {
                    log::error!("CRITICAL: Failed to init HID API: {}", e);
                    return;
                }
            };

            let mut last_physical_check = std::time::Instant::now();
            let check_interval = std::time::Duration::from_millis(1000);

            loop {
                let now = std::time::Instant::now();
                if now.duration_since(last_physical_check) >= check_interval {
                    last_physical_check = now;

                    if let Err(e) = api.refresh_devices() {
                        log::warn!("Failed to refresh USB device list: {}", e);
                    }

                    let local_physical_device = find_device_info(&api);
                    let is_physically_connected = local_physical_device.is_some();

                    let mut clear_backend = false;

                    if let Some(current_backend) = backend.as_mut() {
                        match current_backend {
                            TransportBackend::Local { .. } => {
                                if !is_physically_connected {
                                    log::warn!("DAC physically disconnected (local backend)");
                                    clear_backend = true;
                                }
                            }
                            #[cfg(target_os = "linux")]
                            TransportBackend::Elevated { transport, .. } => {
                                match transport.round_trip(&HelperRequest::Status) {
                                    Ok(HelperResponse::Status {
                                        connected,
                                        physically_present,
                                        ..
                                    }) => {
                                        if !connected || !physically_present {
                                            log::warn!(
                                                "DAC disconnected (elevated backend status: connected={}, physically_present={})",
                                                connected,
                                                physically_present
                                            );
                                            clear_backend = true;
                                        }
                                    }
                                    Ok(_) => {
                                        clear_backend = true;
                                    }
                                    Err(e) => {
                                        log::warn!("Elevated backend status failed: {}", e);
                                        clear_backend = true;
                                    }
                                }
                            }
                        }
                    }

                    if clear_backend {
                        backend = None;
                    }
                }

                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => match cmd {
                        UsbCommand::Connect(target_device, target_backend, resp) => {
                            let preferred = target_backend.unwrap_or(preferred_backend);
                            let result = worker_connect(
                                &mut backend,
                                &mut preferred_backend,
                                &api,
                                Some(preferred),
                                target_device,
                            );
                            let _ = resp.send(result);
                        }
                        UsbCommand::Disconnect(resp) => {
                            if let Some(current) = backend.as_mut() {
                                #[cfg(target_os = "linux")]
                                if let TransportBackend::Elevated { transport, .. } = current {
                                    let _ = transport.round_trip(&HelperRequest::Disconnect);
                                }
                            }
                            backend = None;
                            let _ = resp.send(OperationResult {
                                success: true,
                                data: None,
                                error: None,
                            });
                        }
                        UsbCommand::Status(resp) => {
                            let status = worker_status(&mut backend, &mut api);
                            let _ = resp.send(status);
                        }
                        UsbCommand::PullPEQ(resp) => {
                            let result = worker_pull_peq(&mut backend);
                            let _ = resp.send(result);
                        }
                        UsbCommand::PushPEQ(payload, resp) => {
                            let result = worker_push_peq(&mut backend, payload);
                            let _ = resp.send(result);
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        UsbWorker { tx }
    }

    pub fn connect(&self, device: Option<DeviceInfo>, backend: Option<BackendKind>) -> mpsc::Receiver<ConnectionResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Connect(device, backend, tx));
        rx
    }

    pub fn disconnect(&self) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Disconnect(tx));
        rx
    }

    pub fn status(&self) -> mpsc::Receiver<WorkerStatus> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Status(tx));
        rx
    }

    pub fn pull_peq(&self) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::PullPEQ(tx));
        rx
    }

    pub fn push_peq(&self, payload: PushPayload) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::PushPEQ(payload, tx));
        rx
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
    }
}

fn worker_status(backend: &mut Option<TransportBackend>, api: &mut hidapi::HidApi) -> WorkerStatus {
    let _ = api.refresh_devices();
    let available_devices = crate::hardware::hid::list_devices(api);
    let physically_present = !available_devices.is_empty();

    let mut should_clear_backend = false;

    let status = match backend.as_mut() {
        Some(TransportBackend::Local { info, .. }) => WorkerStatus {
            connected: true,
            physically_present,
            device: Some(info.clone()),
            available_devices: available_devices.clone(),
        },
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info }) => {
            match transport.round_trip(&HelperRequest::Status) {
                Ok(HelperResponse::Status {
                    connected,
                    physically_present,
                    device,
                }) => {
                    if !connected {
                        should_clear_backend = true;
                    }
                    WorkerStatus {
                        connected,
                        physically_present,
                        device: device.or_else(|| Some(info.clone())),
                        available_devices: available_devices.clone(),
                    }
                }
                Ok(_) | Err(_) => {
                    should_clear_backend = true;
                    WorkerStatus {
                        connected: false,
                        physically_present,
                        device: available_devices.first().cloned(),
                        available_devices: available_devices.clone(),
                    }
                }
            }
        }
        None => WorkerStatus {
            connected: false,
            physically_present,
            device: available_devices.first().cloned(),
            available_devices: available_devices.clone(),
        },
    };

    if should_clear_backend {
        *backend = None;
    }

    status
}
