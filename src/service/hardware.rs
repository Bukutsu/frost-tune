// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
use crate::hardware::worker::{BackendKind, UsbWorker, WorkerStatus};
use std::sync::Arc;
#[derive(Clone)]
pub struct HardwareService {
    worker: Arc<UsbWorker>,
}

impl std::fmt::Debug for HardwareService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HardwareService").finish()
    }
}

impl HardwareService {
    /// Creates a new HardwareService, initializing the background USB worker thread.
    pub fn new() -> Self {
        Self {
            worker: Arc::new(UsbWorker::new()),
        }
    }

    /// Attempts to connect to a specific USB DAC device using the specified backend.
    pub async fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> Result<ConnectionResult, String> {
        self.worker.connect(device, backend).await
    }

    /// Disconnects from the current USB DAC device.
    pub async fn disconnect(&self) -> Result<OperationResult, String> {
        self.worker.disconnect().await
    }

    /// Polls the current worker/connection status.
    pub async fn status(&self) -> Result<WorkerStatus, String> {
        self.worker.status().await
    }

    /// Pulls the active PEQ state/filters from the connected USB DAC device.
    pub async fn pull_peq(&self) -> Result<OperationResult, String> {
        self.worker.pull_peq().await
    }

    /// Pushes a new PEQ configuration to the connected USB DAC device.
    pub async fn push_peq(&self, payload: PushPayload) -> Result<OperationResult, String> {
        self.worker.push_peq(payload).await
    }
}

impl Default for HardwareService {
    fn default() -> Self {
        Self::new()
    }
}
