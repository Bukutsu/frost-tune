use serde::{Deserialize, Serialize};
use crate::models::constants::{TP35_VENDOR_ID, TP35_PRODUCT_ID};
use crate::hardware::protocol::{DeviceProtocol, TP35ProProtocol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    TP35Pro,
    Unknown,
}

impl Device {
    pub fn protocol(&self) -> Option<Box<dyn DeviceProtocol>> {
        match self {
            Device::TP35Pro => Some(Box::new(TP35ProProtocol)),
            Device::Unknown => None,
        }
    }

    pub fn from_vid_pid(vid: u16, pid: u16) -> Self {
        match (vid, pid) {
            (TP35_VENDOR_ID, TP35_PRODUCT_ID) => Device::TP35Pro,
            _ => Device::Unknown,
        }
    }

    pub fn vendor_id(&self) -> u16 {
        match self {
            Device::TP35Pro => TP35_VENDOR_ID,
            Device::Unknown => 0,
        }
    }

    pub fn product_id(&self) -> u16 {
        match self {
            Device::TP35Pro => TP35_PRODUCT_ID,
            Device::Unknown => 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Device::TP35Pro => "EPZ TP35 Pro",
            Device::Unknown => "Unknown Device",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub manufacturer: Option<String>,
}
