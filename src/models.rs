use serde::{Deserialize, Serialize};

const TP35_VENDOR_ID: u16 = 0x3302;
const TP35_PRODUCT_ID: u16 = 0x43E6;

pub const VENDOR_ID: u16 = TP35_VENDOR_ID;
pub const PRODUCT_ID: u16 = TP35_PRODUCT_ID;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterType {
    #[serde(rename = "LSQ")]
    LowShelf = 1,
    #[serde(rename = "PK")]
    Peak = 2,
    #[serde(rename = "HSQ")]
    HighShelf = 3,
}

impl From<u8> for FilterType {
    fn from(value: u8) -> Self {
        match value {
            1 => FilterType::LowShelf,
            2 => FilterType::Peak,
            3 => FilterType::HighShelf,
            _ => FilterType::Peak,
        }
    }
}

impl From<FilterType> for u8 {
    fn from(ft: FilterType) -> Self {
        ft as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    pub index: u8,
    pub enabled: bool,
    pub freq: u16,
    pub gain: f64,
    pub q: f64,
    #[serde(rename = "type")]
    pub filter_type: FilterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PEQData {
    pub filters: Vec<Filter>,
    #[serde(rename = "globalGain")]
    pub global_gain: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub device: Option<DeviceInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    pub filters: Vec<Filter>,
    pub global_gain: Option<i8>,
}

impl Filter {
    pub fn enabled(index: u8, enabled: bool) -> Self {
        Filter {
            index,
            enabled,
            freq: 100,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Peak,
        }
    }
}
