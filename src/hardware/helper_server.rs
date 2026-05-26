// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

#[cfg(target_os = "linux")]
use crate::core::{DeviceInfo, Filter, PEQData, PushPayload};
#[cfg(target_os = "linux")]
use crate::hardware::hid::{device_info_from_hid, find_device_info};
use crate::hardware::DeviceProfile;

#[cfg(target_os = "linux")]
use crate::error::{AppError, ErrorKind};
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{
    HelperRequest, HelperResponse, IpcRequest, IpcResponse, IPC_VERSION,
};
#[cfg(target_os = "linux")]
use std::io::{self, BufRead, Read, Write};

#[cfg(target_os = "linux")]
fn pull_logic(
    device: &hidapi::HidDevice,
    profile: &'static dyn DeviceProfile,
    strict: bool,
) -> crate::error::Result<PEQData> {
    let proto = profile.protocol();
    let num_bands = profile.capabilities().num_bands;
    let dummy_check = || false;
    crate::hardware::pipeline::pull_with_retry(
        device,
        proto.as_ref(),
        strict,
        num_bands,
        &dummy_check,
    )
}

#[cfg(target_os = "linux")]
fn push_logic(
    device: &hidapi::HidDevice,
    profile: &'static dyn DeviceProfile,
    filters: Vec<Filter>,
    global_gain: Option<i8>,
) -> crate::error::Result<PEQData> {
    let proto = profile.protocol();
    let caps = profile.capabilities();
    let num_bands = caps.num_bands;

    let mut payload = PushPayload {
        filters,
        global_gain,
    };

    // Helper-side validation: NEVER trust the UI process blindly.
    // Ensure the payload is clamped and valid for the hardware.
    payload.clamp(
        caps.freq_range,
        caps.band_gain_range,
        caps.q_range,
        caps.global_gain_range,
    );
    payload
        .is_valid(
            num_bands,
            caps.freq_range,
            caps.band_gain_range,
            caps.q_range,
            caps.global_gain_range,
        )
        .map_err(|e| AppError::new(ErrorKind::InvalidPayload, e))?;

    let dummy_check = || false;
    crate::hardware::pipeline::push_with_verify(
        device,
        profile,
        proto.as_ref(),
        payload,
        &dummy_check,
    )
}

#[cfg(target_os = "linux")]
fn handle_reset(
    device: &hidapi::HidDevice,
    profile: &'static dyn DeviceProfile,
) -> crate::error::Result<PEQData> {
    let proto = profile.protocol();
    let dummy_check = || false;
    crate::hardware::pipeline::reset_with_verify(device, profile, proto.as_ref(), &dummy_check)
}

#[cfg(target_os = "linux")]
fn require_device(
    device: &Option<hidapi::HidDevice>,
) -> Result<&hidapi::HidDevice, HelperResponse> {
    device.as_ref().ok_or_else(|| HelperResponse::Error {
        error: AppError::new(ErrorKind::NotConnected, "Not connected"),
    })
}

#[cfg(target_os = "linux")]
fn handle_connect(
    api: &mut hidapi::HidApi,
    target: Option<DeviceInfo>,
    device: &mut Option<hidapi::HidDevice>,
    device_info: &mut Option<DeviceInfo>,
    device_profile: &mut Option<&'static dyn DeviceProfile>,
) -> HelperResponse {
    let _ = api.refresh_devices();

    let found_device = if let Some(target) = target {
        api.device_list()
            .find(|d| {
                d.vendor_id() == target.vendor_id
                    && d.product_id() == target.product_id
                    && d.path().to_string_lossy() == target.path
            })
            .cloned()
    } else {
        find_device_info(api)
    };

    match found_device {
        Some(ref found) => {
            if let Some(profile) =
                crate::hardware::get_profile(found.vendor_id(), found.product_id())
            {
                match found.open_device(api) {
                    Ok(opened) => {
                        let info = device_info_from_hid(found);
                        *device = Some(opened);
                        *device_profile = Some(profile);
                        *device_info = Some(info.clone());
                        HelperResponse::Connected { device: Some(info) }
                    }
                    Err(e) => HelperResponse::Error {
                        error: AppError::new(ErrorKind::PermissionDenied, e.to_string()),
                    },
                }
            } else {
                HelperResponse::Error {
                    error: AppError::new(ErrorKind::HardwareError, "Unsupported DAC device"),
                }
            }
        }
        None => HelperResponse::Error {
            error: AppError::new(
                ErrorKind::NotConnected,
                "Device not found. Is it plugged in?",
            ),
        },
    }
}

#[cfg(target_os = "linux")]
fn write_response(
    stdout: &mut io::StdoutLock<'_>,
    response: &IpcResponse,
) -> crate::error::Result<()> {
    let line = serde_json::to_string(response).map_err(|e| {
        AppError::new(
            ErrorKind::ParseError,
            format!("Failed to serialize response: {}", e),
        )
    })?;
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
    let mut api = hidapi::HidApi::new()
        .map_err(|e| AppError::general(format!("Failed to init HID API: {}", e)))?;
    let mut device: Option<hidapi::HidDevice> = None;
    let mut device_info: Option<DeviceInfo> = None;
    let mut device_profile: Option<&'static dyn DeviceProfile> = None;

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
                    &IpcResponse {
                        id: 0,
                        payload: HelperResponse::Error {
                            error: AppError::new(
                                ErrorKind::IpcError,
                                format!("Failed reading request: {}", e),
                            ),
                        },
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
                &IpcResponse {
                    id: 0,
                    payload: HelperResponse::Error {
                        error: AppError::new(ErrorKind::IpcError, "Request payload too large"),
                    },
                },
            );
            break; // Abort connection on oversized payload to prevent OOM
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<IpcRequest>(line) {
            Ok(r) => r,
            Err(e) => {
                let _ = write_response(
                    &mut stdout_lock,
                    &IpcResponse {
                        id: 0,
                        payload: HelperResponse::Error {
                            error: AppError::new(
                                ErrorKind::ParseError,
                                format!("Invalid request payload: {}", e),
                            ),
                        },
                    },
                );
                continue;
            }
        };

        let request_id = request.id;
        let response_payload: HelperResponse = match request.payload {
            HelperRequest::Connect { device: target } => {
                if device.is_some() {
                    HelperResponse::Connected {
                        device: device_info.clone(),
                    }
                } else {
                    handle_connect(
                        &mut api,
                        target,
                        &mut device,
                        &mut device_info,
                        &mut device_profile,
                    )
                }
            }
            HelperRequest::Disconnect => {
                device = None;
                device_info = None;
                device_profile = None;
                HelperResponse::Disconnected
            }
            HelperRequest::Status => {
                let _ = api.refresh_devices();
                let physical = find_device_info(&api);
                let physically_present = physical.is_some();
                if device.is_some() && !physically_present {
                    device = None;
                    device_info = None;
                    device_profile = None;
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
            HelperRequest::Ping => HelperResponse::Pong,
            HelperRequest::PullPeq { strict } => match require_device(&device) {
                Ok(d) => match device_profile {
                    Some(dp) => match pull_logic(d, dp, strict) {
                        Ok(peq) => match serde_json::to_value(peq) {
                            Ok(value) => HelperResponse::Pulled { data: value },
                            Err(e) => HelperResponse::Error {
                                error: AppError::new(
                                    ErrorKind::ParseError,
                                    format!("Serialization failed: {}", e),
                                ),
                            },
                        },
                        Err(e) => HelperResponse::Error { error: e },
                    },
                    None => HelperResponse::Error {
                        error: AppError::new(ErrorKind::NotConnected, "Device profile not loaded"),
                    },
                },
                Err(payload) => payload,
            },
            HelperRequest::ResetPeq => match require_device(&device) {
                Ok(d) => match device_profile {
                    Some(dp) => match handle_reset(d, dp) {
                        Ok(peq) => match serde_json::to_value(peq) {
                            Ok(value) => HelperResponse::Pulled { data: value },
                            Err(e) => HelperResponse::Error {
                                error: AppError::new(
                                    ErrorKind::ParseError,
                                    format!("Serialization failed: {}", e),
                                ),
                            },
                        },
                        Err(e) => HelperResponse::Error { error: e },
                    },
                    None => HelperResponse::Error {
                        error: AppError::new(ErrorKind::NotConnected, "Device profile not loaded"),
                    },
                },
                Err(payload) => payload,
            },
            HelperRequest::PushPeq {
                filters,
                global_gain,
            } => match require_device(&device) {
                Ok(d) => match device_profile {
                    Some(dp) => match push_logic(d, dp, filters, global_gain) {
                        Ok(peq) => match serde_json::to_value(peq) {
                            Ok(value) => HelperResponse::Pushed { data: value },
                            Err(e) => HelperResponse::Error {
                                error: AppError::new(
                                    ErrorKind::ParseError,
                                    format!("Serialization failed: {}", e),
                                ),
                            },
                        },
                        Err(e) => HelperResponse::Error { error: e },
                    },
                    None => HelperResponse::Error {
                        error: AppError::new(ErrorKind::NotConnected, "Device profile not loaded"),
                    },
                },
                Err(payload) => payload,
            },
            HelperRequest::Shutdown => {
                write_response(
                    &mut stdout_lock,
                    &IpcResponse {
                        id: request_id,
                        payload: HelperResponse::Ok,
                    },
                )?;
                break;
            }
        };

        write_response(
            &mut stdout_lock,
            &IpcResponse {
                id: request_id,
                payload: response_payload,
            },
        )?;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn run() -> Result<(), String> {
    Err("helper server is only available on Linux".to_string())
}
