use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::hardware::hid::{delay_ms, device_info_from_hid, find_device_info};
use crate::hardware::operations::{compare_peq, pull_peq_data, rollback_and_verify};
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain, WriteTiming,
};
use crate::hardware::protocol::DeviceProtocol;
use crate::error::{AppError, ErrorKind, Result as AppResult};
use crate::models::{
    ConnectionResult, Device, DeviceInfo, OperationResult, PEQData, PushPayload,
};

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
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Local,
    #[cfg(target_os = "linux")]
    Elevated,
}

enum TransportBackend {
    Local {
        device: hidapi::HidDevice,
        device_type: Device,
        info: DeviceInfo,
    },
    #[cfg(target_os = "linux")]
    Elevated {
        transport: ElevatedTransport,
        info: DeviceInfo,
    },
}

impl TransportBackend {
    fn device_info(&self) -> DeviceInfo {
        match self {
            TransportBackend::Local { info, .. } => info.clone(),
            #[cfg(target_os = "linux")]
            TransportBackend::Elevated { info, .. } => info.clone(),
        }
    }
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
                    let mut has_logical_connection = backend.is_some();

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
                        has_logical_connection = false;
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

fn worker_connect(
    backend: &mut Option<TransportBackend>,
    preferred_backend: &mut BackendKind,
    api: &hidapi::HidApi,
    preferred: Option<BackendKind>,
    target_device: Option<DeviceInfo>,
) -> ConnectionResult {
    if let Some(current_backend) = backend.as_ref() {
        return ConnectionResult {
            success: true,
            device: Some(current_backend.device_info()),
            error: None,
        };
    }

    #[cfg(target_os = "linux")]
    let local_first = !matches!(preferred, Some(BackendKind::Elevated));

    #[cfg(not(target_os = "linux"))]
    let local_first = true;

    if local_first {
        let local = try_connect_local(api, target_device.clone());
        match local {
            Ok(Some(connected)) => {
                *preferred_backend = BackendKind::Local;
                let info = connected.device_info();
                *backend = Some(connected);
                return ConnectionResult {
                    success: true,
                    device: Some(info),
                    error: None,
                };
            }
            Ok(None) => {
                return ConnectionResult {
                    success: false,
                    device: None,
                    error: Some(AppError::new(ErrorKind::NotConnected, "Device not found. Is it plugged in?")),
                };
            }
            Err(local_err) => {
                #[cfg(target_os = "linux")]
                {
                    if local_err.kind == ErrorKind::PermissionDenied {
                        return ConnectionResult {
                            success: false,
                            device: target_device,
                            error: Some(AppError::new(ErrorKind::PolkitAuthRequired, "Authentication required to access USB DAC on Linux.")),
                        };
                    }
                }

                return ConnectionResult {
                    success: false,
                    device: None,
                    error: Some(local_err),
                };
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        match try_connect_elevated() {
            Ok(connected) => {
                *preferred_backend = BackendKind::Elevated;
                let info = connected.device_info();
                *backend = Some(connected);
                return ConnectionResult {
                    success: true,
                    device: Some(info),
                    error: None,
                };
            }
            Err(elevated_err) => {
                let local = try_connect_local(api, target_device.clone());
                match local {
                    Ok(Some(connected)) => {
                        *preferred_backend = BackendKind::Local;
                        let info = connected.device_info();
                        *backend = Some(connected);
                        return ConnectionResult {
                            success: true,
                            device: Some(info),
                            error: None,
                        };
                    }
                    Ok(None) => {
                        return ConnectionResult {
                            success: false,
                            device: None,
                            error: Some(AppError::new(ErrorKind::NotConnected, "Device not found. Is it plugged in?")),
                        };
                    }
                    Err(_local_err) => {
                        return ConnectionResult {
                            success: false,
                            device: None,
                            error: Some(elevated_err),
                        };
                    }
                }
            }
        }
    }

    #[allow(unreachable_code)]
    ConnectionResult {
        success: false,
        device: None,
        error: Some(AppError::new(ErrorKind::Unknown, "Connect failed")),
    }
}

fn try_connect_local(api: &hidapi::HidApi, target_device: Option<DeviceInfo>) -> AppResult<Option<TransportBackend>> {
    let device_info = match target_device {
        Some(info) => {
            // Find the matching hidapi::DeviceInfo
            match api.device_list().find(|d| d.path().to_string_lossy() == info.path) {
                Some(d) => d.clone(),
                None => return Ok(None),
            }
        }
        None => match crate::hardware::hid::find_device_info(api) {
            Some(d) => d,
            None => return Ok(None),
        }
    };

    let info = device_info_from_hid(&device_info);
    match device_info.open_device(api) {
        Ok(d) => {
            let device_type =
                Device::from_vid_pid(device_info.vendor_id(), device_info.product_id());
            Ok(Some(TransportBackend::Local {
                device: d,
                device_type,
                info,
            }))
        }
        Err(e) => Err(AppError::new(ErrorKind::PermissionDenied, e.to_string())),
    }
}

#[cfg(target_os = "linux")]
fn try_connect_elevated() -> AppResult<TransportBackend> {
    let mut transport = ElevatedTransport::spawn()?;
    match transport.round_trip(&HelperRequest::Connect)? {
        HelperResponse::Connected { device } => {
            let info = device.ok_or_else(|| AppError::general("Helper connected without device info"))?;
            Ok(TransportBackend::Elevated { transport, info })
        }
        HelperResponse::Error { error } => Err(error),
        _ => Err(AppError::general("Unexpected response from elevated helper during connect")),
    }
}

fn worker_pull_peq(backend: &mut Option<TransportBackend>) -> OperationResult {
    match backend.as_mut() {
        Some(TransportBackend::Local {
            device,
            device_type,
            info: _,
        }) => {
            if let Some(proto) = device_type.protocol() {
                worker_pull_peq_local(device, proto.as_ref())
            } else {
                OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::HardwareError, "Unsupported device protocol")),
                }
            }
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PullPeq { strict: false }) {
                Ok(HelperResponse::Pulled { data }) => {
                    let peq = serde_json::from_value::<PEQData>(data).ok();
                    let success = peq.is_some();
                    OperationResult {
                        success,
                        error: if !success { Some(AppError::new(ErrorKind::ParseError, "Failed to parse data from helper")) } else { None },
                        data: peq,
                    }
                }
                Ok(HelperResponse::Error { error }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(error),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::HardwareError, "Unexpected helper response for pull")),
                },
                Err(e) => {
                    *backend = None;
                    OperationResult {
                        success: false,
                        data: None,
                        error: Some(e),
                    }
                }
            }
        }
        None => OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
        },
    }
}

fn worker_push_peq(
    backend: &mut Option<TransportBackend>,
    payload: PushPayload,
) -> OperationResult {
    match backend.as_mut() {
        Some(TransportBackend::Local {
            device,
            device_type,
            info: _,
        }) => {
            if let Some(proto) = device_type.protocol() {
                worker_push_peq_local(device, proto.as_ref(), payload)
            } else {
                OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::HardwareError, "Unsupported device protocol")),
                }
            }
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PushPeq {
                filters: payload.filters,
                global_gain: payload.global_gain,
            }) {
                Ok(HelperResponse::Pushed { data }) => {
                    let peq = serde_json::from_value::<PEQData>(data).ok();
                    let success = peq.is_some();
                    OperationResult {
                        success,
                        error: if !success { Some(AppError::new(ErrorKind::ParseError, "Failed to parse data from helper")) } else { None },
                        data: peq,
                    }
                }
                Ok(HelperResponse::Error { error }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(error),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::HardwareError, "Unexpected helper response for push")),
                },
                Err(e) => {
                    *backend = None;
                    OperationResult {
                        success: false,
                        data: None,
                        error: Some(e),
                    }
                }
            }
        }
        None => OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
        },
    }
}


fn worker_pull_peq_local(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
) -> OperationResult {
    let first_result = pull_peq_data(device, proto, false);

    let needs_retry = match &first_result {
        Ok(peq) => {
            let all_disabled = peq.filters.iter().all(|f| !f.enabled);
            let has_default_gain = peq.global_gain == 0;
            let all_default_freq = peq.filters.iter().all(|f| f.freq == 100);
            all_disabled && has_default_gain && all_default_freq
        }
        Err(_) => true,
    };

    let final_result = if needs_retry {
        delay_ms(100);
        pull_peq_data(device, proto, false)
    } else {
        first_result
    };

    match final_result {
        Ok(peq) => OperationResult {
            success: true,
            data: Some(peq),
            error: None,
        },
        Err(e) => OperationResult {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}


fn worker_push_peq_local(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    payload: PushPayload,
) -> OperationResult {
    let mut payload = payload;
    payload.clamp();
    if let Err(e) = payload.is_valid() {
        return OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(ErrorKind::ParseError, e)),
        };
    }

    let snapshot = match pull_peq_data(device, proto, true) {
        Ok(s) => s,
        Err(e) => {
            return OperationResult {
                success: false,
                data: None,
                error: Some(e),
            }
        }
    };

    let timing = WriteTiming::default();
    let write_res = (|| -> crate::error::Result<()> {
        init_device_session(device, proto)?;
        write_filters_and_gain(
            device,
            proto,
            &payload.filters,
            payload.global_gain.unwrap_or(0),
            &timing,
        )?;
        commit_changes(device, proto, &timing)?;
        Ok(())
    })();

    if let Err(e) = write_res {
        if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
            return OperationResult {
                success: false,
                data: None,
                error: Some(AppError::new(
                    ErrorKind::RollbackFailed,
                    format!("Write failed: {} | rollback failed: {}", e.message, rollback_error.message),
                )),
            };
        }
        return OperationResult {
            success: false,
            data: None,
            error: Some(e),
        };
    }

    for attempt in 0..3 {
        let backoff_ms = 200 * (2u64.pow(attempt as u32));
        delay_ms(backoff_ms as u64);
        match pull_peq_data(device, proto, true) {
            Ok(read_back) => {
                if compare_peq(
                    &read_back,
                    &payload.filters,
                    payload.global_gain.unwrap_or(0),
                )
                .is_ok()
                {
                    return OperationResult {
                        success: true,
                        data: Some(read_back),
                        error: None,
                    };
                }
            }
            Err(e) => {
                if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
                    return OperationResult {
                        success: false,
                        data: None,
                        error: Some(AppError::new(
                            ErrorKind::RollbackFailed,
                            format!("Verify read error: {} | rollback failed: {}", e.message, rollback_error.message),
                        )),
                    };
                }
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(e),
                };
            }
        }
    }

    if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
        return OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(
                ErrorKind::RollbackFailed,
                format!("Verification failed: settings did not match | rollback failed: {}", rollback_error.message),
            )),
        };
    }

    OperationResult {
        success: false,
        data: None,
        error: Some(AppError::new(ErrorKind::VerifyFailed, "Verification failed: settings did not match")),
    }
}
