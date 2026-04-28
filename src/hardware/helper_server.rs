#[cfg(target_os = "linux")]
use crate::hardware::hid::{delay_ms, device_info_from_hid, find_device_info};
#[cfg(target_os = "linux")]
use crate::hardware::operations::{compare_peq, pull_peq_data, rollback_and_verify};
#[cfg(target_os = "linux")]
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain, WriteTiming,
};
#[cfg(target_os = "linux")]
use crate::models::{Device, DeviceInfo, Filter, PEQData, PushPayload};

#[cfg(target_os = "linux")]
use crate::error::{AppError, ErrorKind};
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse, IPC_VERSION};
#[cfg(target_os = "linux")]
use std::io::{self, BufRead, Write};





#[cfg(target_os = "linux")]
fn pull_logic(
    device: &hidapi::HidDevice,
    device_type: Device,
    strict: bool,
) -> crate::error::Result<PEQData> {
    let proto = device_type.protocol().ok_or_else(|| AppError::new(ErrorKind::HardwareError, "Unsupported device protocol"))?;
    let first_result = pull_peq_data(device, proto.as_ref(), strict);

    let needs_retry = match &first_result {
        Ok(peq) => {
            let all_disabled = peq.filters.iter().all(|f| !f.enabled);
            let has_default_gain = peq.global_gain == 0;
            let all_default_freq = peq.filters.iter().all(|f| f.freq == 100);
            all_disabled && has_default_gain && all_default_freq
        }
        Err(_) => true,
    };

    if needs_retry {
        delay_ms(100);
        pull_peq_data(device, proto.as_ref(), strict)
    } else {
        first_result
    }
}

#[cfg(target_os = "linux")]
fn push_logic(
    device: &hidapi::HidDevice,
    device_type: Device,
    filters: Vec<Filter>,
    global_gain: Option<i8>,
) -> crate::error::Result<PEQData> {
    let proto = device_type.protocol().ok_or_else(|| AppError::new(ErrorKind::HardwareError, "Unsupported device protocol"))?;

    let mut payload = PushPayload {
        filters,
        global_gain,
    };
    payload.clamp();
    payload.is_valid().map_err(|e| AppError::new(ErrorKind::ParseError, e))?;

    let snapshot = pull_peq_data(device, proto.as_ref(), true)?;

    let timing = WriteTiming::default();
    let write_res = (|| -> crate::error::Result<()> {
        init_device_session(device, proto.as_ref())?;
        write_filters_and_gain(
            device,
            proto.as_ref(),
            &payload.filters,
            payload.global_gain.unwrap_or(0),
            &timing,
        )?;
        commit_changes(device, proto.as_ref(), &timing)?;
        Ok(())
    })();

    if let Err(e) = write_res {
        if let Err(rollback_error) = rollback_and_verify(device, proto.as_ref(), &snapshot) {
            return Err(AppError::new(
                ErrorKind::RollbackFailed,
                format!("Write failed: {} | rollback failed: {}", e.message, rollback_error.message),
            ));
        }
        return Err(e);
    }

    for attempt in 0..3 {
        let backoff_ms = 200 * (2u64.pow(attempt as u32));
        delay_ms(backoff_ms as u64);
        match pull_peq_data(device, proto.as_ref(), true) {
            Ok(read_back) => {
                if compare_peq(
                    &read_back,
                    &payload.filters,
                    payload.global_gain.unwrap_or(0),
                )
                .is_ok()
                {
                    return Ok(read_back);
                }
            }
            Err(e) => {
                if let Err(rollback_error) = rollback_and_verify(device, proto.as_ref(), &snapshot) {
                    return Err(AppError::new(
                        ErrorKind::RollbackFailed,
                        format!("Verify read error: {} | rollback failed: {}", e.message, rollback_error.message),
                    ));
                }
                return Err(e);
            }
        }
    }

    if let Err(rollback_error) = rollback_and_verify(device, proto.as_ref(), &snapshot) {
        return Err(AppError::new(
            ErrorKind::RollbackFailed,
            format!("Verification failed | rollback failed: {}", rollback_error.message),
        ));
    }

    Err(AppError::new(ErrorKind::VerifyFailed, "Verification failed: settings did not match"))
}

#[cfg(target_os = "linux")]
fn write_response(
    stdout: &mut io::StdoutLock<'_>,
    response: &HelperResponse,
) -> crate::error::Result<()> {
    let line = serde_json::to_string(response)
        .map_err(|e| AppError::new(ErrorKind::ParseError, format!("Failed to serialize response: {}", e)))?;
    stdout
        .write_all(line.as_bytes())
        .map_err(|e| AppError::general(format!("Failed writing response: {}", e)))?;
    stdout
        .write_all(b"\n")
        .map_err(|e| AppError::general(format!("Failed writing response delimiter: {}", e)))?;
    stdout
        .flush()
        .map_err(|e| AppError::general(format!("Failed flushing response: {}", e)))?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn run() -> crate::error::Result<()> {
    let mut api = hidapi::HidApi::new().map_err(|e| AppError::general(format!("Failed to init HID API: {}", e)))?;
    let mut device: Option<hidapi::HidDevice> = None;
    let mut device_info: Option<DeviceInfo> = None;
    let mut device_type: Device = Device::Unknown;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let lines = stdin.lock().lines();
    let mut stdout_lock = stdout.lock();

    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                let _ = write_response(
                    &mut stdout_lock,
                    &HelperResponse::Error {
                        error: AppError::general(format!("Failed reading request: {}", e)),
                    },
                );
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<HelperRequest>(&line) {
            Ok(r) => r,
            Err(e) => {
                let _ = write_response(
                    &mut stdout_lock,
                    &HelperResponse::Error {
                        error: AppError::general(format!("Invalid request payload: {}", e)),
                    },
                );
                continue;
            }
        };

        let response: HelperResponse = match request {
            HelperRequest::Connect => {
                if device.is_some() {
                    HelperResponse::Connected {
                        device: device_info.clone(),
                    }
                } else {
                    let _ = api.refresh_devices();
                    match find_device_info(&api) {
                        Some(found) => {
                            let found_type =
                                Device::from_vid_pid(found.vendor_id(), found.product_id());
                            if found_type == Device::Unknown {
                                HelperResponse::Error {
                                    error: AppError::new(ErrorKind::HardwareError, "Unsupported DAC device"),
                                }
                            } else {
                                match found.open_device(&api) {
                                    Ok(opened) => {
                                        let info = device_info_from_hid(&found);
                                        device = Some(opened);
                                        device_type = found_type;
                                        device_info = Some(info.clone());
                                        HelperResponse::Connected { device: Some(info) }
                                    }
                                    Err(e) => HelperResponse::Error {
                                        error: AppError::new(ErrorKind::PermissionDenied, e.to_string()),
                                    },
                                }
                            }
                        }
                        None => HelperResponse::Error {
                            error: AppError::new(ErrorKind::NotConnected, "Device not found. Is it plugged in?"),
                        },
                    }
                }
            }
            HelperRequest::Disconnect => {
                device = None;
                device_info = None;
                device_type = Device::Unknown;
                HelperResponse::Disconnected
            }
            HelperRequest::Status => {
                let _ = api.refresh_devices();
                let physical = find_device_info(&api);
                let physically_present = physical.is_some();
                if device.is_some() && !physically_present {
                    device = None;
                    device_info = None;
                    device_type = Device::Unknown;
                }
                HelperResponse::Status {
                    connected: device.is_some(),
                    physically_present,
                    device: physical.as_ref().map(device_info_from_hid),
                }
            }
            HelperRequest::Version => HelperResponse::Version {
                version: IPC_VERSION.to_string(),
            },
            HelperRequest::PullPeq { strict } => {
                if let Some(d) = &device {
                    match pull_logic(d, device_type, strict) {
                        Ok(peq) => match serde_json::to_value(peq) {
                            Ok(value) => HelperResponse::Pulled { data: value },
                            Err(e) => HelperResponse::Error {
                                error: AppError::new(ErrorKind::ParseError, format!("Serialization failed: {}", e)),
                            },
                        },
                        Err(e) => HelperResponse::Error { error: e },
                    }
                } else {
                    HelperResponse::Error {
                        error: AppError::new(ErrorKind::NotConnected, "Not connected"),
                    }
                }
            }
            HelperRequest::PushPeq {
                filters,
                global_gain,
            } => {
                if let Some(d) = &device {
                    match push_logic(d, device_type, filters, global_gain) {
                        Ok(peq) => match serde_json::to_value(peq) {
                            Ok(value) => HelperResponse::Pushed { data: value },
                            Err(e) => HelperResponse::Error {
                                error: AppError::new(ErrorKind::ParseError, format!("Serialization failed: {}", e)),
                            },
                        },
                        Err(e) => HelperResponse::Error { error: e },
                    }
                } else {
                    HelperResponse::Error {
                        error: AppError::new(ErrorKind::NotConnected, "Not connected"),
                    }
                }
            }
            HelperRequest::Shutdown => {
                write_response(&mut stdout_lock, &HelperResponse::Ok)?;
                break;
            }
        };

        write_response(&mut stdout_lock, &response)?;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn run() -> Result<(), String> {
    Err("helper server is only available on Linux".to_string())
}
