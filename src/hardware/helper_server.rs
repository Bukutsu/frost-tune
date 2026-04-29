#[cfg(target_os = "linux")]
use crate::hardware::hid::{device_info_from_hid, find_device_info};
#[cfg(target_os = "linux")]
use crate::models::{Device, DeviceInfo, Filter, PEQData, PushPayload};

#[cfg(target_os = "linux")]
use crate::error::{AppError, ErrorKind};
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse, IPC_VERSION};
#[cfg(target_os = "linux")]
use std::io::{self, BufRead, Read, Write};





#[cfg(target_os = "linux")]
fn pull_logic(
    device: &hidapi::HidDevice,
    device_type: Device,
    strict: bool,
) -> crate::error::Result<PEQData> {
    let proto = device_type.protocol().ok_or_else(|| AppError::new(ErrorKind::HardwareError, "Unsupported device protocol"))?;
    crate::hardware::pipeline::pull_with_retry(device, proto.as_ref(), strict)
}

#[cfg(target_os = "linux")]
fn push_logic(
    device: &hidapi::HidDevice,
    device_type: Device,
    filters: Vec<Filter>,
    global_gain: Option<i8>,
) -> crate::error::Result<PEQData> {
    let proto = device_type.protocol().ok_or_else(|| AppError::new(ErrorKind::HardwareError, "Unsupported device protocol"))?;
    let payload = PushPayload {
        filters,
        global_gain,
    };
    crate::hardware::pipeline::push_with_verify(device, proto.as_ref(), payload)
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
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    loop {
        let mut line = String::new();
        let bytes_read = match stdin_lock.by_ref().take(65536).read_line(&mut line) {
            Ok(n) => n,
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

        if bytes_read == 0 {
            break;
        }

        if bytes_read == 65536 && !line.ends_with('\n') {
            let _ = write_response(
                &mut stdout_lock,
                &HelperResponse::Error {
                    error: AppError::general("Request payload too large"),
                },
            );
            let mut buf = vec![];
            let _ = stdin_lock.read_until(b'\n', &mut buf);
            continue;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<HelperRequest>(line) {
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
