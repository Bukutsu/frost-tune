use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::hardware::hid::{find_device_info, pull_peq_internal, delay_ms};
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload, PEQData, Filter};
use crate::hardware::packet_builder::{commit_changes, write_filters_and_gain, WriteTiming, init_device_session};

#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub connected: bool,
    pub physically_present: bool,
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
                // Check hotplug every 1 second (matching old project pattern)
                let now = std::time::Instant::now();
                if now.duration_since(last_physical_check) >= check_interval {
                    last_physical_check = now;
                    
                    if let Err(e) = api.refresh_devices() {
                        log::warn!("Failed to refresh USB device list: {}", e);
                    }

                    let is_physically_connected = find_device_info(&api).is_some();
                    let is_logically_connected = device.is_some();

                    log::debug!("Hotplug check: logical={}, physical={}", is_logically_connected, is_physically_connected);

                    // If logically connected but physically lost -> disconnect
                    if is_logically_connected && !is_physically_connected {
                        log::warn!("DAC physically disconnected!");
                        device = None;
                    } 
                    // If logically disconnected but physically present -> auto-reconnect
                    else if !is_logically_connected && is_physically_connected {
                        log::info!("DAC detected, auto-connecting...");
                        let res = worker_connect(&mut device, &api);
                        if res.success {
                            log::info!("Auto-reconnect successful");
                        } else {
                            log::warn!("Auto-reconnect failed: {:?}", res.error);
                        }
                    }
                }

                // Process commands (non-blocking like old project)
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => {
                        match cmd {
                            UsbCommand::Connect(resp) => {
                                let result = worker_connect(&mut device, &api);
                                let _ = resp.send(result);
                            }
                            UsbCommand::Disconnect(resp) => {
                                device = None;
                                let _ = resp.send(OperationResult { success: true, data: None, error: None });
                            }
                            UsbCommand::Status(resp) => {
                                let _ = api.refresh_devices();
                                let physically_present = find_device_info(&api).is_some();
                                let status = WorkerStatus {
                                    connected: device.is_some(),
                                    physically_present,
                                };
                                let _ = resp.send(status);
                            }
                            UsbCommand::PullPEQ(resp) => {
                                let result = worker_pull_peq(&device);
                                let _ = resp.send(result);
                            }
                            UsbCommand::PushPEQ(payload, resp) => {
                                let result = worker_push_peq(&device, payload);
                                let _ = resp.send(result);
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // No command, continue to next hotplug check
                    }
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
    fn default() -> Self { Self::new() }
}

fn worker_connect(device: &mut Option<hidapi::HidDevice>, api: &hidapi::HidApi) -> ConnectionResult {
    if device.is_some() {
        log::info!("Already connected");
        return ConnectionResult { success: true, device: None, error: None };
    }

    log::info!("Scanning for device {:04x}:{:04x}", crate::models::VENDOR_ID, crate::models::PRODUCT_ID);

    let device_info = match find_device_info(api) {
        Some(d) => d,
        None => {
            log::warn!("Device not found - not connected or different VID/PID");
            return ConnectionResult { success: false, device: None, error: Some("Device not found. Is it plugged in?".into()) };
        }
    };

    log::info!("Device found, attempting to open: {:?}", device_info.path());

    match device_info.open_device(api) {
        Ok(d) => {
            *device = Some(d);
            log::info!("Successfully connected to device");
            ConnectionResult {
                success: true,
                device: Some(DeviceInfo {
                    vendor_id: device_info.vendor_id(),
                    product_id: device_info.product_id(),
                    path: device_info.path().to_string_lossy().into(),
                    manufacturer: device_info.manufacturer_string().map(|s| s.to_string()),
                }),
                error: None,
            }
        }
        Err(e) => {
            let err_str = e.to_string();
            log::error!("Failed to open device: {}", err_str);

            let friendly_error = if err_str.contains("Access denied") || err_str.contains("Permission denied") {
                "Access denied. Check USB permissions (udev rule needed on Linux)".into()
            } else if err_str.contains("busy") || err_str.contains("in use") {
                "Device is busy. Another app may be connected.".into()
            } else if err_str.contains("No such file") || err_str.contains("No such device") {
                "Device disconnected during open. Try reconnecting.".into()
            } else {
                format!("Open failed: {}", err_str)
            };

            ConnectionResult { success: false, device: None, error: Some(friendly_error) }
        }
    }
}

fn pull_peq_data(d: &hidapi::HidDevice, strict: bool) -> Result<PEQData, String> {
    let mut last_err = "Timeout".to_string();
    for attempt in 0..3 {
        match pull_peq_internal(d, strict) {
            Ok(data) => return Ok(data),
            Err(e) => { last_err = e; }
        }
        if attempt < 2 {
            delay_ms(200);
        }
    }
    Err(last_err)
}

fn worker_pull_peq(device: &Option<hidapi::HidDevice>) -> OperationResult {
    let d = match device { Some(d) => d, None => return OperationResult { success: false, data: None, error: Some("Not connected".into()) } };
    log::info!("Pulling PEQ data from device...");
    
    // First attempt
    let first_result = pull_peq_data(d, false);
    
    // Check for suspicious (default/empty) response
    let needs_retry = match &first_result {
        Ok(peq) => {
            let all_disabled = peq.filters.iter().all(|f| !f.enabled);
            let has_default_gain = peq.global_gain == 0;
            let has_no_filters = peq.filters.is_empty();
            // Suspicious if: all disabled + zero gain + no filters (likely default read)
            all_disabled && has_default_gain && has_no_filters
        }
        Err(_) => true, // Retry on any error
    };
    
    // Retry once if suspicious
    let final_result = if needs_retry {
        log::warn!("Pull returned suspicious result, retrying...");
        delay_ms(100);
        pull_peq_data(d, false)
    } else {
        first_result
    };
    
    match final_result {
        Ok(peq_data) => {
            let filter_count = peq_data.filters.len();
            let gain = peq_data.global_gain;
            log::info!("Pull successful: {} filters, global_gain: {}", filter_count, gain);
            OperationResult { success: true, data: Some(serde_json::to_value(peq_data).unwrap()), error: None }
        }
        Err(e) => {
            log::error!("Pull failed: {}", e);
            OperationResult { success: false, data: None, error: Some(e) }
        }
    }
}

pub fn compare_peq(actual: &PEQData, filters: &[Filter], gain: i8) -> Result<(), String> {
    if actual.global_gain != gain {
        return Err(format!("Global gain mismatch: expected {}, got {}", gain, actual.global_gain));
    }
    for (a, f) in actual.filters.iter().zip(filters.iter()) {
        if f.enabled != a.enabled {
            return Err(format!("Band {} enabled state mismatch", f.index));
        }
        if f.enabled && (a.gain - f.gain).abs() > 0.15 {
            return Err(format!("Band {} gain mismatch: expected {:.2}, got {:.2}", f.index, f.gain, a.gain));
        }
        if f.enabled && (a.freq as i32 - f.freq as i32).abs() > 0 {
            return Err(format!("Band {} freq mismatch: expected {}, got {}", f.index, f.freq, a.freq));
        }
        if f.enabled && (a.q - f.q).abs() > 0.05 {
            return Err(format!("Band {} Q mismatch: expected {:.2}, got {:.2}", f.index, f.q, a.q));
        }
        if f.enabled && f.filter_type != a.filter_type {
            return Err(format!("Band {} filter type mismatch", f.index));
        }
    }
    Ok(())
}

fn worker_push_peq(device: &Option<hidapi::HidDevice>, payload: PushPayload) -> OperationResult {
    let d = match device { Some(d) => d, None => return OperationResult { success: false, data: None, error: Some("Not connected".into()) } };

    let mut payload = payload;
    payload.clamp();
    if let Err(e) = payload.is_valid() {
        return OperationResult { success: false, data: None, error: Some(e) };
    }
    
    let snapshot = match pull_peq_data(d, true) { Ok(s) => s, Err(e) => return OperationResult { success: false, data: None, error: Some(e) } };
    
    let timing = WriteTiming::default();
    if let Err(e) = (|| -> Result<(), String> {
        init_device_session(d)?;
        write_filters_and_gain(d, &payload.filters, payload.global_gain.unwrap_or(0), &timing)?;
        commit_changes(d, &timing)?;
        Ok(())
    })() {
        let rollback_err = rollback_state(d, &snapshot);
        if rollback_err.is_err() {
            return OperationResult { success: false, data: None, error: Some("Write failed: rollback also failed".into()) };
        }
        return OperationResult { success: false, data: None, error: Some(format!("Write failed: {}", e)) };
    }

    for attempt in 0..3 {
        let backoff_ms = 300 + (attempt * 200);
        delay_ms(backoff_ms);
        match pull_peq_data(d, true) {
            Ok(read_back) => {
                if compare_peq(&read_back, &payload.filters, payload.global_gain.unwrap_or(0)).is_ok() {
                    return OperationResult { success: true, data: Some(serde_json::to_value(read_back).unwrap()), error: None };
                }
            }
            Err(e) => return OperationResult { success: false, data: None, error: Some(format!("Verify read error: {}", e)) },
        }
    }
    let rollback_err = rollback_state(d, &snapshot);
    if rollback_err.is_err() {
        return OperationResult { success: false, data: None, error: Some("Verify failed: rollback also failed".into()) };
    }
    OperationResult { success: false, data: None, error: Some("Verification failed: settings did not match".into()) }
}

fn rollback_state(d: &hidapi::HidDevice, state: &PEQData) -> Result<(), String> {
    let timing = WriteTiming::default();
    write_filters_and_gain(d, &state.filters, state.global_gain, &timing)?;
    commit_changes(d, &timing)
}