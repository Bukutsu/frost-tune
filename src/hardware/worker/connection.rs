use crate::hardware::hid::{device_info_from_hid, find_device_info};
use crate::error::{AppError, ErrorKind, Result as AppResult};
use crate::models::{ConnectionResult, Device, DeviceInfo};
use crate::hardware::worker::backend::{BackendKind, TransportBackend};

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

pub fn worker_connect(
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

pub fn try_connect_local(api: &hidapi::HidApi, target_device: Option<DeviceInfo>) -> AppResult<Option<TransportBackend>> {
    let device_info = match target_device {
        Some(info) => {
            match api.device_list().find(|d| d.path().to_string_lossy() == info.path) {
                Some(d) => d.clone(),
                None => return Ok(None),
            }
        }
        None => match find_device_info(api) {
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
pub fn try_connect_elevated() -> AppResult<TransportBackend> {
    let mut transport = ElevatedTransport::spawn()?;
    match transport.round_trip(&HelperRequest::Connect)? {
        HelperResponse::Connected { device } => {
            let info = device.ok_or_else(|| AppError::new(ErrorKind::IpcError, "Helper connected without device info"))?;
            Ok(TransportBackend::Elevated { transport, info })
        }
        HelperResponse::Error { error, .. } => Err(error),
        _ => Err(AppError::new(ErrorKind::IpcError, "Unexpected response from elevated helper during connect")),
    }
}
