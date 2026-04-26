use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::hardware::hid::{delay_ms, find_device_info, pull_peq_internal};
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain, WriteTiming,
};
use crate::hardware::protocol::DeviceProtocol;
use crate::models::{
    ConnectionResult, Device, DeviceInfo, Filter, OperationResult, PEQData, PushPayload,
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
}

fn device_info_from_hid(device_info: &hidapi::DeviceInfo) -> DeviceInfo {
    DeviceInfo {
        vendor_id: device_info.vendor_id(),
        product_id: device_info.product_id(),
        path: device_info.path().to_string_lossy().into(),
        manufacturer: device_info.manufacturer_string().map(|s| s.to_string()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendKind {
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
    Connect(mpsc::Sender<ConnectionResult>),
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
            let mut manual_disconnect: bool = false;
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
                        manual_disconnect = false;
                        has_logical_connection = false;
                    }

                    if !has_logical_connection && is_physically_connected && !manual_disconnect {
                        log::info!("DAC detected, auto-connecting...");
                        let preferred = preferred_backend;
                        let res = worker_connect(
                            &mut backend,
                            &mut preferred_backend,
                            &api,
                            Some(preferred),
                        );
                        if res.success {
                            log::info!("Auto-reconnect successful");
                        } else {
                            log::warn!("Auto-reconnect failed: {:?}", res.error);
                        }
                    } else if !has_logical_connection && !is_physically_connected {
                        manual_disconnect = false;
                    }
                }

                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => match cmd {
                        UsbCommand::Connect(resp) => {
                            manual_disconnect = false;
                            let preferred = preferred_backend;
                            let result = worker_connect(
                                &mut backend,
                                &mut preferred_backend,
                                &api,
                                Some(preferred),
                            );
                            let _ = resp.send(result);
                        }
                        UsbCommand::Disconnect(resp) => {
                            manual_disconnect = true;
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

    pub fn connect(&self) -> mpsc::Receiver<ConnectionResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Connect(tx));
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
    let physical_device = find_device_info(api);
    let physically_present = physical_device.is_some();

    let mut should_clear_backend = false;

    let status = match backend.as_mut() {
        Some(TransportBackend::Local { info, .. }) => WorkerStatus {
            connected: true,
            physically_present,
            device: Some(info.clone()),
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
                    }
                }
                Ok(_) | Err(_) => {
                    should_clear_backend = true;
                    WorkerStatus {
                        connected: false,
                        physically_present,
                        device: physical_device.as_ref().map(device_info_from_hid),
                    }
                }
            }
        }
        None => WorkerStatus {
            connected: false,
            physically_present,
            device: physical_device.as_ref().map(device_info_from_hid),
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
) -> ConnectionResult {
    if let Some(current_backend) = backend.as_ref() {
        return ConnectionResult {
            success: true,
            device: Some(current_backend.device_info()),
            error: None,
        };
    }

    let local_first = !matches!(preferred, Some(BackendKind::Elevated));

    if local_first {
        let local = try_connect_local(api);
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
                    error: Some("Device not found. Is it plugged in?".into()),
                };
            }
            Err(local_err) => {
                #[cfg(target_os = "linux")]
                {
                    if is_permission_denied_error(&local_err) {
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
                                return ConnectionResult {
                                    success: false,
                                    device: None,
                                    error: Some(format!("POLKIT_AUTH_REQUIRED: {}", elevated_err)),
                                };
                            }
                        }
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
                let local = try_connect_local(api);
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
                            error: Some("Device not found. Is it plugged in?".into()),
                        };
                    }
                    Err(local_err) => {
                        return ConnectionResult {
                            success: false,
                            device: None,
                            error: Some(format!(
                                "POLKIT_AUTH_REQUIRED: {} | local open failed: {}",
                                elevated_err, local_err
                            )),
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
        error: Some("Connect failed".into()),
    }
}

fn try_connect_local(api: &hidapi::HidApi) -> Result<Option<TransportBackend>, String> {
    let device_info = match find_device_info(api) {
        Some(d) => d,
        None => return Ok(None),
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
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(target_os = "linux")]
fn try_connect_elevated() -> Result<TransportBackend, String> {
    let mut transport = ElevatedTransport::spawn()?;
    match transport.round_trip(&HelperRequest::Connect)? {
        HelperResponse::Connected { device } => {
            let info = device.ok_or_else(|| "Helper connected without device info".to_string())?;
            Ok(TransportBackend::Elevated { transport, info })
        }
        HelperResponse::Error { message } => Err(message),
        _ => Err("Unexpected response from elevated helper during connect".to_string()),
    }
}

fn worker_pull_peq(backend: &mut Option<TransportBackend>) -> OperationResult {
    match backend.as_mut() {
        Some(TransportBackend::Local {
            device,
            device_type,
            info: _,
        }) => {
            let proto = device_type.protocol();
            worker_pull_peq_local(device, proto.as_ref())
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PullPeq { strict: false }) {
                Ok(HelperResponse::Pulled { data }) => OperationResult {
                    success: true,
                    data: Some(data),
                    error: None,
                },
                Ok(HelperResponse::Error { message }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(message),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some("Unexpected helper response for pull".to_string()),
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
            error: Some("Not connected".into()),
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
            let proto = device_type.protocol();
            worker_push_peq_local(device, proto.as_ref(), payload)
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PushPeq {
                filters: payload.filters,
                global_gain: payload.global_gain,
            }) {
                Ok(HelperResponse::Pushed { data }) => OperationResult {
                    success: true,
                    data: Some(data),
                    error: None,
                },
                Ok(HelperResponse::Error { message }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(message),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some("Unexpected helper response for push".to_string()),
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
            error: Some("Not connected".into()),
        },
    }
}

fn pull_peq_data(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    strict: bool,
) -> Result<PEQData, String> {
    let mut last_err = "Timeout".to_string();
    for attempt in 0..3 {
        match pull_peq_internal(d, proto, strict) {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_err = e;
            }
        }
        if attempt < 2 {
            delay_ms(200);
        }
    }
    Err(last_err)
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
        Ok(peq_data) => match serde_json::to_value(peq_data) {
            Ok(val) => OperationResult {
                success: true,
                data: Some(val),
                error: None,
            },
            Err(e) => OperationResult {
                success: false,
                data: None,
                error: Some(format!("Serialization failed: {}", e)),
            },
        },
        Err(e) => OperationResult {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

pub fn compare_peq(actual: &PEQData, filters: &[Filter], gain: i8) -> Result<(), String> {
    if actual.global_gain != gain {
        return Err(format!(
            "Global gain mismatch: expected {}, got {}",
            gain, actual.global_gain
        ));
    }
    for (a, f) in actual.filters.iter().zip(filters.iter()) {
        if (a.gain - f.gain).abs() > 0.15 {
            return Err(format!(
                "Band {} gain mismatch: expected {:.2}, got {:.2}",
                f.index, f.gain, a.gain
            ));
        }
        if (a.freq as i32 - f.freq as i32).abs() > 0 {
            return Err(format!(
                "Band {} freq mismatch: expected {}, got {}",
                f.index, f.freq, a.freq
            ));
        }
        if (a.q - f.q).abs() > 0.05 {
            return Err(format!(
                "Band {} Q mismatch: expected {:.2}, got {:.2}",
                f.index, f.q, a.q
            ));
        }
        if f.filter_type != a.filter_type {
            return Err(format!("Band {} filter type mismatch", f.index));
        }
    }
    Ok(())
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
            error: Some(e),
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
    if let Err(e) = (|| -> Result<(), String> {
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
    })() {
        if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
            return OperationResult {
                success: false,
                data: None,
                error: Some(format!(
                    "Write failed: {} | rollback failed: {}",
                    e, rollback_error
                )),
            };
        }
        return OperationResult {
            success: false,
            data: None,
            error: Some(format!("Write failed: {}", e)),
        };
    }

    for attempt in 0..3 {
        let backoff_ms = 300 + (attempt * 200);
        delay_ms(backoff_ms);
        match pull_peq_data(device, proto, true) {
            Ok(read_back) => {
                if compare_peq(
                    &read_back,
                    &payload.filters,
                    payload.global_gain.unwrap_or(0),
                )
                .is_ok()
                {
                    match serde_json::to_value(read_back) {
                        Ok(val) => {
                            return OperationResult {
                                success: true,
                                data: Some(val),
                                error: None,
                            };
                        }
                        Err(e) => {
                            return OperationResult {
                                success: false,
                                data: None,
                                error: Some(format!("Serialization error: {}", e)),
                            };
                        }
                    }
                }
            }
            Err(e) => {
                if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
                    return OperationResult {
                        success: false,
                        data: None,
                        error: Some(format!(
                            "Verify read error: {} | rollback failed: {}",
                            e, rollback_error
                        )),
                    };
                }
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(format!("Verify read error: {}", e)),
                };
            }
        }
    }
    if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
        return OperationResult {
            success: false,
            data: None,
            error: Some(format!(
                "Verification failed: settings did not match | rollback failed: {}",
                rollback_error
            )),
        };
    }
    OperationResult {
        success: false,
        data: None,
        error: Some("Verification failed: settings did not match".into()),
    }
}

fn rollback_and_verify(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    snapshot: &PEQData,
) -> Result<(), String> {
    rollback_state(d, proto, snapshot).map_err(|e| format!("rollback write failed: {}", e))?;

    let restored =
        pull_peq_data(d, proto, true).map_err(|e| format!("rollback verify read failed: {}", e))?;

    compare_peq(&restored, &snapshot.filters, snapshot.global_gain)
        .map_err(|e| format!("rollback verify mismatch: {}", e))
}

fn rollback_state(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    state: &PEQData,
) -> Result<(), String> {
    let timing = WriteTiming::default();
    write_filters_and_gain(d, proto, &state.filters, state.global_gain, &timing)?;
    commit_changes(d, proto, &timing)
}

#[cfg(target_os = "linux")]
fn is_permission_denied_error(message: &str) -> bool {
    let lowered = message.to_lowercase();
    lowered.contains("permission denied") || lowered.contains("access denied")
}
