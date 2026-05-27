// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

#[cfg(target_os = "linux")]
use crate::core::{DeviceInfo, Filter, PEQData};
#[cfg(target_os = "linux")]
use crate::hardware::device_io::{DiscoveryProvider, PhysicalInterface};
#[cfg(target_os = "linux")]
use crate::hardware::DeviceProfile;
use crate::hardware::PushPayload;

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
    device: &dyn PhysicalInterface,
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
    device: &dyn PhysicalInterface,
    profile: &'static dyn DeviceProfile,
    filters: Vec<Filter>,
    global_gain: Option<i8>,
    skip_verify: bool,
) -> crate::error::Result<PEQData> {
    let proto = profile.protocol();
    let caps = profile.capabilities();

    let payload = PushPayload::new_validated(filters, global_gain, &caps)
        .map_err(|e| AppError::new(ErrorKind::InvalidPayload, e))?;

    let dummy_check = || false;
    crate::hardware::pipeline::push_with_verify(
        device,
        profile,
        proto.as_ref(),
        payload,
        skip_verify,
        &dummy_check,
    )
}

#[cfg(target_os = "linux")]
fn handle_reset(
    device: &dyn PhysicalInterface,
    profile: &'static dyn DeviceProfile,
) -> crate::error::Result<PEQData> {
    let proto = profile.protocol();
    let dummy_check = || false;
    crate::hardware::pipeline::reset_with_verify(device, profile, proto.as_ref(), &dummy_check)
}

#[cfg(target_os = "linux")]
fn require_device(
    device: &Option<Box<dyn PhysicalInterface>>,
) -> Result<&dyn PhysicalInterface, HelperResponse> {
    device.as_deref().ok_or_else(|| HelperResponse::Error {
        error: AppError::new(ErrorKind::NotConnected, "Not connected"),
    })
}

#[cfg(target_os = "linux")]
fn handle_connect(
    target: Option<DeviceInfo>,
    device: &mut Option<Box<dyn PhysicalInterface>>,
    device_info: &mut Option<DeviceInfo>,
    device_profile: &mut Option<&'static dyn DeviceProfile>,
) -> HelperResponse {
    let provider = crate::hardware::hid::HidDiscoveryProvider;
    let resolved_target = if let Some(target) = target {
        Some(target)
    } else {
        match provider.list_devices() {
            Ok(devices) => devices.first().cloned(),
            Err(e) => return HelperResponse::Error { error: e },
        }
    };

    let target = match resolved_target {
        Some(t) => t,
        None => {
            return HelperResponse::Error {
                error: AppError::new(
                    ErrorKind::NotConnected,
                    "Device not found. Is it plugged in?",
                ),
            };
        }
    };

    if let Some(profile) = crate::hardware::get_profile(target.vendor_id, target.product_id) {
        match provider.open_device(&target) {
            Ok(opened) => {
                *device = Some(opened);
                *device_profile = Some(profile);
                *device_info = Some(target.clone());
                HelperResponse::Connected {
                    device: Some(target),
                }
            }
            Err(e) => HelperResponse::Error { error: e },
        }
    } else {
        HelperResponse::Error {
            error: AppError::new(ErrorKind::HardwareError, "Unsupported DAC device"),
        }
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
pub fn run(ipc_token: String) -> crate::error::Result<()> {
    let provider = crate::hardware::hid::HidDiscoveryProvider;
    let mut device: Option<Box<dyn PhysicalInterface>> = None;
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
                break;
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
                break;
            }
        };

        let request_id = request.id;

        if request.auth != ipc_token {
            let _ = write_response(
                &mut stdout_lock,
                &IpcResponse {
                    id: request_id,
                    payload: HelperResponse::Error {
                        error: AppError::new(ErrorKind::IpcError, "Authentication token mismatch"),
                    },
                },
            );
            break;
        }

        let response_payload: HelperResponse = match request.payload {
            HelperRequest::Connect { device: target } => {
                if device.is_some() {
                    HelperResponse::Connected {
                        device: device_info.clone(),
                    }
                } else {
                    handle_connect(target, &mut device, &mut device_info, &mut device_profile)
                }
            }
            HelperRequest::Disconnect => {
                device = None;
                device_info = None;
                device_profile = None;
                HelperResponse::Disconnected
            }
            HelperRequest::Status => {
                let (available_devices, list_err) = match provider.list_devices() {
                    Ok(devices) => (devices, None),
                    Err(e) => {
                        log::error!("HID device enumeration failed: {}", e.message);
                        (vec![], Some(e))
                    }
                };
                let physically_present = if list_err.is_some() {
                    // Cannot enumerate — assume device is still present to avoid
                    // false-positive disconnects on transient HID API failures.
                    device.is_some()
                } else if let Some(ref current) = device_info {
                    available_devices.iter().any(|d| d.path == current.path)
                } else {
                    !available_devices.is_empty()
                };

                if device.is_some() && !physically_present {
                    device = None;
                    device_info = None;
                    device_profile = None;
                }
                HelperResponse::Status {
                    connected: device.is_some(),
                    physically_present,
                    device: device_info.clone(),
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
                skip_verify,
            } => match require_device(&device) {
                Ok(d) => match device_profile {
                    Some(dp) => match push_logic(d, dp, filters, global_gain, skip_verify) {
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
                let _ = write_response(
                    &mut stdout_lock,
                    &IpcResponse {
                        id: request_id,
                        payload: HelperResponse::Ok,
                    },
                );
                break;
            }
        };

        if write_response(
            &mut stdout_lock,
            &IpcResponse {
                id: request_id,
                payload: response_payload,
            },
        )
        .is_err()
        {
            break;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn run(_ipc_token: String) -> Result<(), String> {
    Err("helper server is only available on Linux".to_string())
}
