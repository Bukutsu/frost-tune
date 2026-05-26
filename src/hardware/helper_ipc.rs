// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::{DeviceInfo, Filter};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

pub const IPC_VERSION: &str = "1.4.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub auth: String,
    pub id: u64,
    #[serde(flatten)]
    pub payload: HelperRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: u64,
    #[serde(flatten)]
    pub payload: HelperResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum HelperRequest {
    Connect {
        device: Option<DeviceInfo>,
    },
    Disconnect,
    Status,
    Version,
    Ping,
    PullPeq {
        strict: bool,
    },
    PushPeq {
        filters: Vec<Filter>,
        global_gain: Option<i8>,
        skip_verify: bool,
    },
    ResetPeq,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
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
    Pong,
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
