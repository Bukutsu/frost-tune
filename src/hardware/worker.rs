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
            let mut device: Option<hidapi::HidDevice> = None;
            let mut device_type = Device::Unknown;
            let mut manual_disconnect = false;
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

                    let is_physically_connected = find_device_info(&api).is_some();
                    let is_logically_connected = device.is_some();

                    if is_logically_connected && !is_physically_connected {
                        log::warn!("DAC physically disconnected!");
                        device = None;
                        device_type = Device::Unknown;
                        manual_disconnect = false;
                    } else if !is_logically_connected
                        && is_physically_connected
                        && !manual_disconnect
                    {
                        log::info!("DAC detected, auto-connecting...");
                        let res = worker_connect(&mut device, &mut device_type, &api);
                        if res.success {
                            log::info!("Auto-reconnect successful");
                        } else {
                            log::warn!("Auto-reconnect failed: {:?}", res.error);
                        }
                    } else if !is_logically_connected && !is_physically_connected {
                        manual_disconnect = false;
                    }
                }

                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => match cmd {
                        UsbCommand::Connect(resp) => {
                            manual_disconnect = false;
                            let result = worker_connect(&mut device, &mut device_type, &api);
                            let _ = resp.send(result);
                        }
                        UsbCommand::Disconnect(resp) => {
                            manual_disconnect = true;
                            device = None;
                            device_type = Device::Unknown;
                            let _ = resp.send(OperationResult {
                                success: true,
                                data: None,
                                error: None,
                            });
                        }
                        UsbCommand::Status(resp) => {
                            let _ = api.refresh_devices();
                            let physical_device = find_device_info(&api);
                            let physically_present = physical_device.is_some();
                            let status = WorkerStatus {
                                connected: device.is_some(),
                                physically_present,
                                device: physical_device.as_ref().map(device_info_from_hid),
                            };
                            let _ = resp.send(status);
                        }
                        UsbCommand::PullPEQ(resp) => {
                            let proto = device_type.protocol();
                            let result = worker_pull_peq(&device, proto.as_ref());
                            let _ = resp.send(result);
                        }
                        UsbCommand::PushPEQ(payload, resp) => {
                            let proto = device_type.protocol();
                            let result = worker_push_peq(&device, proto.as_ref(), payload);
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

fn worker_connect(
    device: &mut Option<hidapi::HidDevice>,
    device_type: &mut Device,
    api: &hidapi::HidApi,
) -> ConnectionResult {
    if device.is_some() {
        let current = find_device_info(api).map(|d| device_info_from_hid(&d));
        return ConnectionResult {
            success: true,
            device: current,
            error: None,
        };
    }

    let device_info = match find_device_info(api) {
        Some(d) => d,
        None => {
            return ConnectionResult {
                success: false,
                device: None,
                error: Some("Device not found. Is it plugged in?".into()),
            };
        }
    };

    match device_info.open_device(api) {
        Ok(d) => {
            *device = Some(d);
            *device_type = Device::from_vid_pid(device_info.vendor_id(), device_info.product_id());
            ConnectionResult {
                success: true,
                device: Some(device_info_from_hid(&device_info)),
                error: None,
            }
        }
        Err(e) => ConnectionResult {
            success: false,
            device: None,
            error: Some(e.to_string()),
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

fn worker_pull_peq(
    device: &Option<hidapi::HidDevice>,
    proto: &dyn DeviceProtocol,
) -> OperationResult {
    let d = match device {
        Some(d) => d,
        None => {
            return OperationResult {
                success: false,
                data: None,
                error: Some("Not connected".into()),
            }
        }
    };

    let first_result = pull_peq_data(d, proto, false);

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
        pull_peq_data(d, proto, false)
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

fn worker_push_peq(
    device: &Option<hidapi::HidDevice>,
    proto: &dyn DeviceProtocol,
    payload: PushPayload,
) -> OperationResult {
    let d = match device {
        Some(d) => d,
        None => {
            return OperationResult {
                success: false,
                data: None,
                error: Some("Not connected".into()),
            }
        }
    };

    let mut payload = payload;
    payload.clamp();
    if let Err(e) = payload.is_valid() {
        return OperationResult {
            success: false,
            data: None,
            error: Some(e),
        };
    }

    let snapshot = match pull_peq_data(d, proto, true) {
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
        init_device_session(d, proto)?;
        write_filters_and_gain(
            d,
            proto,
            &payload.filters,
            payload.global_gain.unwrap_or(0),
            &timing,
        )?;
        commit_changes(d, proto, &timing)?;
        Ok(())
    })() {
        if let Err(rollback_error) = rollback_and_verify(d, proto, &snapshot) {
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
        match pull_peq_data(d, proto, true) {
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
                if let Err(rollback_error) = rollback_and_verify(d, proto, &snapshot) {
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
    if let Err(rollback_error) = rollback_and_verify(d, proto, &snapshot) {
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
