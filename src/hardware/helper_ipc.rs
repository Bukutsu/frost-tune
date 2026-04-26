use crate::models::{DeviceInfo, Filter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HelperRequest {
    Connect,
    Disconnect,
    Status,
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
    Pulled {
        data: serde_json::Value,
    },
    Pushed {
        data: serde_json::Value,
    },
    Error {
        message: String,
    },
    Ok,
}
