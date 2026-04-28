use crate::error::AppError;
use crate::models::{DeviceInfo, Filter};
use serde::{Deserialize, Serialize};

pub const IPC_VERSION: &str = "1.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HelperRequest {
    Connect,
    Disconnect,
    Status,
    Version,
    PullPeq {
        strict: bool,
    },
    PushPeq {
        filters: Vec<Filter>,
        global_gain: Option<i8>,
    },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HelperResponse {
    Connected {
        device: Option<DeviceInfo>,
    },
    Disconnected,
    Status {
        connected: bool,
        physically_present: bool,
        device: Option<DeviceInfo>,
    },
    Version {
        version: String,
    },
    Pulled {
        data: serde_json::Value,
    },
    Pushed {
        data: serde_json::Value,
    },
    Error {
        error: AppError,
    },
    Ok,
}
