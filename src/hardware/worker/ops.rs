use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::backend::TransportBackend;
use crate::models::{OperationResult, PEQData, PushPayload};

#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

pub fn worker_pull_peq(backend: &mut Option<TransportBackend>) -> OperationResult {
    match backend.as_mut() {
        Some(TransportBackend::Local {
            device,
            device_type,
            info: _,
        }) => {
            if let Some(proto) = device_type.protocol() {
                worker_pull_peq_local(device, proto.as_ref())
            } else {
                OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Unsupported device protocol",
                    )),
                }
            }
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PullPeq { strict: false }) {
                Ok(HelperResponse::Pulled { data }) => {
                    let peq = serde_json::from_value::<PEQData>(data).ok();
                    let success = peq.is_some();
                    OperationResult {
                        success,
                        error: if !success {
                            Some(AppError::new(
                                ErrorKind::ParseError,
                                "Failed to parse data from helper",
                            ))
                        } else {
                            None
                        },
                        data: peq,
                    }
                }
                Ok(HelperResponse::Error { error, .. }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(error),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Unexpected helper response for pull",
                    )),
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
            error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
        },
    }
}

pub fn worker_push_peq(
    backend: &mut Option<TransportBackend>,
    payload: PushPayload,
) -> OperationResult {
    match backend.as_mut() {
        Some(TransportBackend::Local {
            device,
            device_type,
            info: _,
        }) => {
            if let Some(proto) = device_type.protocol() {
                worker_push_peq_local(device, proto.as_ref(), payload)
            } else {
                OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Unsupported device protocol",
                    )),
                }
            }
        }
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info: _ }) => {
            match transport.round_trip(&HelperRequest::PushPeq {
                filters: payload.filters,
                global_gain: payload.global_gain,
            }) {
                Ok(HelperResponse::Pushed { data }) => {
                    let peq = serde_json::from_value::<PEQData>(data).ok();
                    let success = peq.is_some();
                    OperationResult {
                        success,
                        error: if !success {
                            Some(AppError::new(
                                ErrorKind::ParseError,
                                "Failed to parse data from helper",
                            ))
                        } else {
                            None
                        },
                        data: peq,
                    }
                }
                Ok(HelperResponse::Error { error, .. }) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(error),
                },
                Ok(_) => OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Unexpected helper response for push",
                    )),
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
            error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
        },
    }
}

fn worker_pull_peq_local(
    device: &hidapi::HidDevice,
    proto: &dyn crate::hardware::protocol::DeviceProtocol,
) -> OperationResult {
    match crate::hardware::pipeline::pull_with_retry(device, proto, false) {
        Ok(peq) => OperationResult {
            success: true,
            data: Some(peq),
            error: None,
        },
        Err(e) => OperationResult {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

fn worker_push_peq_local(
    device: &hidapi::HidDevice,
    proto: &dyn crate::hardware::protocol::DeviceProtocol,
    payload: PushPayload,
) -> OperationResult {
    match crate::hardware::pipeline::push_with_verify(device, proto, payload) {
        Ok(peq) => OperationResult {
            success: true,
            data: Some(peq),
            error: None,
        },
        Err(e) => OperationResult {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}
