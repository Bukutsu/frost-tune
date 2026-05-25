// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::{ConnectionResult, DeviceInfo, OperationResult};
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DisconnectReason {
    #[default]
    None,
    Manual,
    DeviceLost,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ConnectionMessage {
    ConnectPressed(DeviceInfo),
    DisconnectPressed,
    WorkerConnected(ConnectionResult),
    WorkerDisconnected(OperationResult),
    WorkerStatus(WorkerStatus),
    WorkerBackendReset,
}

#[derive(Debug, Clone, Default)]
pub struct OperationLock {
    pub is_pulling: bool,
    pub is_pushing: bool,
    pub is_connecting: bool,
    pub is_disconnecting: bool,
}

#[derive(Default)]
pub struct ConnectionComponent {
    pub status: ConnectionStatus,
    pub disconnect_reason: DisconnectReason,
    pub operation_lock: OperationLock,
    pub worker: Option<Arc<UsbWorker>>,
    pub connected_device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
}
