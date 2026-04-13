use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot};

use crate::error::AppError;
use crate::hid::{find_device_info, pull_peq_internal, delay_ms};
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload, PEQData, Filter};
use crate::packet_builder::{commit_changes, write_filters_and_gain, WriteTiming, init_device_session};

pub enum UsbCommand {
    Connect {
        resp: oneshot::Sender<ConnectionResult>,
    },
    AutoConnect {
        resp: oneshot::Sender<ConnectionResult>,
    },
    Disconnect {
        resp: oneshot::Sender<OperationResult>,
    },
    PullPEQ {
        resp: oneshot::Sender<OperationResult>,
    },
    PushPEQ {
        payload: PushPayload,
        resp: oneshot::Sender<OperationResult>,
    },
}

pub struct UsbWorker {
    pub tx: mpsc::Sender<UsbCommand>,
}

impl UsbWorker {
    pub fn spawn(app_handle: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel::<UsbCommand>(32);

        thread::spawn(move || {
            let mut device: Option<hidapi::HidDevice> = None;
            let api_res = hidapi::HidApi::new();
            if let Err(e) = api_res {
                log::error!("CRITICAL: Failed to initialize HID API: {}", e);
                return;
            }
            let mut api = api_res.unwrap();

            loop {
                match rx.try_recv() {
                    Ok(cmd) => match cmd {
                        UsbCommand::Connect { resp } => {
                            let result = worker_connect(&mut device, &api);
                            let _ = resp.send(result);
                        }
                        UsbCommand::AutoConnect { resp } => {
                            let result = worker_auto_connect(&mut device, &api);
                            let _ = resp.send(result);
                        }
                        UsbCommand::Disconnect { resp } => {
                            device.take();
                            let _ = resp.send(OperationResult {
                                success: true,
                                data: None,
                                error: None,
                            });
                            let _ = app_handle.emit("device-disconnected", ());
                        }
                        UsbCommand::PullPEQ { resp } => {
                            let result = worker_pull_peq(&device);
                            let _ = resp.send(result);
                        }
                        UsbCommand::PushPEQ { payload, resp } => {
                            let result = worker_push_peq(&device, payload);
                            let _ = resp.send(result);
                        }
                    },
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                    Err(mpsc::error::TryRecvError::Empty) => {}
                }

                if let Err(e) = api.refresh_devices() {
                    log::warn!("Failed to refresh USB device list: {}", e);
                }

                let is_physically_connected = find_device_info(&api).is_some();
                let is_logically_connected = device.is_some();

                if is_logically_connected && !is_physically_connected {
                    log::warn!("DAC physically disconnected!");
                    device.take();
                    let _ = app_handle.emit("device-disconnected", ());
                } else if !is_logically_connected && is_physically_connected {
                    log::info!("DAC detected, auto-connecting...");
                    let res = worker_connect(&mut device, &api);
                    if res.success {
                        let _ = app_handle.emit("device-connected", res.device);
                    }
                }

                thread::sleep(Duration::from_millis(1000));
            }
        });

        UsbWorker { tx }
    }
}

fn worker_connect(
    device: &mut Option<hidapi::HidDevice>,
    api: &hidapi::HidApi,
) -> ConnectionResult {
    if device.is_some() {
        return ConnectionResult {
            success: true,
            device: None,
            error: None,
        };
    }

    let device_info = match find_device_info(api) {
        Some(d) => d,
        None => {
            return ConnectionResult {
                success: false,
                device: None,
                error: Some(AppError::DeviceNotFound.to_string()),
            };
        }
    };

    match device_info.open_device(api) {
        Ok(d) => {
            let path = device_info.path().to_string_lossy().into_owned();
            let manufacturer = device_info.manufacturer_string().map(|s| s.to_string());

            let info = DeviceInfo {
                vendor_id: device_info.vendor_id(),
                product_id: device_info.product_id(),
                path,
                manufacturer,
            };

            *device = Some(d);
            log::info!("Connected to TP35 Pro");

            ConnectionResult {
                success: true,
                device: Some(info),
                error: None,
            }
        }
        Err(e) => {
            let err_str = e.to_string();
            let app_err = if err_str.contains("Access denied") || err_str.contains("Permission denied") {
                AppError::PermissionDenied
            } else if err_str.contains("Device or resource busy") {
                AppError::DeviceBusy
            } else {
                AppError::DeviceOpenFailed(err_str)
            };

            ConnectionResult {
                success: false,
                device: None,
                error: Some(app_err.into()),
            }
        }
    }
}

fn worker_auto_connect(
    device: &mut Option<hidapi::HidDevice>,
    api: &hidapi::HidApi,
) -> ConnectionResult {
    if find_device_info(api).is_some() {
        worker_connect(device, api)
    } else {
        ConnectionResult {
            success: false,
            device: None,
            error: Some(AppError::DeviceNotFound.to_string()),
        }
    }
}

fn pull_peq_data(d: &hidapi::HidDevice, strict: bool) -> Result<PEQData, AppError> {
    let mut last_err = AppError::ReadTimeout;
    for attempt in 0..3 {
        if attempt > 0 { delay_ms(200); }
        match pull_peq_internal(d, strict) {
            Ok(data) => return Ok(data),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

fn worker_pull_peq(device: &Option<hidapi::HidDevice>) -> OperationResult {
    let d = match device.as_ref() {
        Some(d) => d,
        None => return OperationResult { success: false, data: None, error: Some(AppError::DeviceNotFound.to_string()) },
    };

    match pull_peq_data(d, false) {
        Ok(peq_data) => {
            let data_json = serde_json::to_value(peq_data).unwrap();
            OperationResult { success: true, data: Some(data_json), error: None }
        }
        Err(e) => OperationResult { success: false, data: None, error: Some(e.into()) },
    }
}

pub fn compare_peq(actual: &PEQData, intended_filters: &[Filter], intended_gain: i8) -> Result<(), String> {
    if actual.global_gain != intended_gain {
        return Err(format!("Gain mismatch: actual={}, intended={}", actual.global_gain, intended_gain));
    }
    for (a, i) in actual.filters.iter().zip(intended_filters.iter()) {
        if a.enabled != i.enabled {
            return Err(format!("Band {} enabled mismatch", i.index));
        }
        if i.enabled {
            if a.freq != i.freq { return Err(format!("Band {} freq mismatch", i.index)); }
            if (a.gain - i.gain).abs() > 0.15 { return Err(format!("Band {} gain mismatch", i.index)); }
            if (a.q - i.q).abs() > 0.15 { return Err(format!("Band {} Q mismatch", i.index)); }
            if a.filter_type != i.filter_type { return Err(format!("Band {} type mismatch", i.index)); }
        }
    }
    Ok(())
}

fn worker_push_peq(device: &Option<hidapi::HidDevice>, payload: PushPayload) -> OperationResult {
    let global_gain = payload.global_gain.unwrap_or(0);
    let filters = payload.filters;
    let d = match device.as_ref() {
        Some(d) => d,
        None => return OperationResult { success: false, data: None, error: Some(AppError::DeviceNotFound.to_string()) },
    };

    // 1. Snapshot
    let snapshot = match pull_peq_data(d, true) {
        Ok(data) => data,
        Err(e) => return OperationResult { success: false, data: None, error: Some(format!("Pre-push snapshot failed: {}", e)) },
    };

    // 2. Push
    let write_timing = WriteTiming::default();
    let push_res = (|| -> Result<(), AppError> {
        init_device_session(d)?;
        write_filters_and_gain(d, &filters, global_gain, &write_timing)?;
        commit_changes(d, &write_timing)?;
        Ok(())
    })();

    if let Err(e) = push_res {
        let _ = rollback_state(d, &snapshot);
        return OperationResult { success: false, data: None, error: Some(e.to_string()) };
    }

    // 3. Verify with retries
    let mut last_verify_error = String::new();
    for attempt in 0..3 {
        let wait = match attempt { 0 => 300, 1 => 600, _ => 1200 };
        delay_ms(wait);
        match pull_peq_data(d, true) {
            Ok(read_back) => {
                if let Err(mismatch) = compare_peq(&read_back, &filters, global_gain) {
                    last_verify_error = mismatch;
                } else {
                    return OperationResult { success: true, data: Some(serde_json::to_value(read_back).unwrap()), error: None };
                }
            }
            Err(e) => last_verify_error = e.to_string(),
        }
    }

    let _ = rollback_state(d, &snapshot);
    OperationResult { success: false, data: None, error: Some(format!("Verification failed: {}", last_verify_error)) }
}

fn rollback_state(d: &hidapi::HidDevice, state: &PEQData) -> Result<(), AppError> {
    log::info!("Rolling back PEQ state");
    let write_timing = WriteTiming::default();
    write_filters_and_gain(d, &state.filters, state.global_gain, &write_timing)?;
    commit_changes(d, &write_timing)?;
    Ok(())
}
