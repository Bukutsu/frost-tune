use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::hardware::hid::{find_device_info, pull_peq_internal, delay_ms};
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload, PEQData, Filter};
use crate::hardware::packet_builder::{commit_changes, write_filters_and_gain, WriteTiming, init_device_session};

pub enum UsbCommand {
    Connect(mpsc::Sender<ConnectionResult>),
    Disconnect(mpsc::Sender<OperationResult>),
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
                Ok(a) => a,
                Err(e) => {
                    log::error!("Failed to init HID API: {}", e);
                    return;
                }
            };

            loop {
                match rx.recv() {
                    Ok(UsbCommand::Connect(resp)) => {
                        let result = worker_connect(&mut device, &api);
                        let _ = resp.send(result);
                    }
                    Ok(UsbCommand::Disconnect(resp)) => {
                        device = None;
                        let _ = resp.send(OperationResult { success: true, data: None, error: None });
                    }
                    Ok(UsbCommand::PullPEQ(resp)) => {
                        let result = worker_pull_peq(&device);
                        let _ = resp.send(result);
                    }
                    Ok(UsbCommand::PushPEQ(payload, resp)) => {
                        let result = worker_push_peq(&device, payload);
                        let _ = resp.send(result);
                    }
                    Err(_) => break,
                }

                let _ = api.refresh_devices();
                thread::sleep(Duration::from_millis(500));
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
        return ConnectionResult { success: true, device: None, error: None };
    }

    let device_info = match find_device_info(api) {
        Some(d) => d,
        None => return ConnectionResult { success: false, device: None, error: Some("Device not found".into()) },
    };

    match device_info.open_device(api) {
        Ok(d) => {
            *device = Some(d);
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
        Err(e) => ConnectionResult { success: false, device: None, error: Some(e.to_string()) },
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
    match pull_peq_data(d, false) {
        Ok(peq_data) => OperationResult { success: true, data: Some(serde_json::to_value(peq_data).unwrap()), error: None },
        Err(e) => OperationResult { success: false, data: None, error: Some(e) },
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
        let _ = rollback_state(d, &snapshot);
        return OperationResult { success: false, data: None, error: Some(e) };
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
            Err(e) => return OperationResult { success: false, data: None, error: Some(e) },
        }
    }
    let _ = rollback_state(d, &snapshot);
    OperationResult { success: false, data: None, error: Some("Verification failed".into()) }
}

fn rollback_state(d: &hidapi::HidDevice, state: &PEQData) -> Result<(), String> {
    let timing = WriteTiming::default();
    write_filters_and_gain(d, &state.filters, state.global_gain, &timing)?;
    commit_changes(d, &timing)
}