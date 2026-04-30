use crate::models::{Device, DeviceInfo};

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Local,
    #[cfg(target_os = "linux")]
    Elevated,
}

pub enum TransportBackend {
    Local {
        device: hidapi::HidDevice,
        device_type: Device,
        info: DeviceInfo,
    },
    #[cfg(target_os = "linux")]
    Elevated {
        transport: ElevatedTransport,
        info: DeviceInfo,
    },
}

impl TransportBackend {
    pub fn device_info(&self) -> DeviceInfo {
        match self {
            TransportBackend::Local { info, .. } => info.clone(),
            #[cfg(target_os = "linux")]
            TransportBackend::Elevated { info, .. } => info.clone(),
        }
    }
}
