// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::hardware::worker::{BackendKind, UsbWorker, WorkerStatus};
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
use std::sync::Arc;
use tokio::sync::oneshot;

/// Facade wrapping the low-level UsbWorker background thread and USB interface.
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
    pub fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> oneshot::Receiver<ConnectionResult> {
        self.worker.connect(device, backend)
    }

    /// Disconnects from the current USB DAC device.
    pub fn disconnect(&self) -> oneshot::Receiver<OperationResult> {
        self.worker.disconnect()
    }

    /// Polls the current worker/connection status.
    pub fn status(&self) -> oneshot::Receiver<WorkerStatus> {
        self.worker.status()
    }

    /// Pulls the active PEQ state/filters from the connected USB DAC device.
    pub fn pull_peq(&self) -> oneshot::Receiver<OperationResult> {
        self.worker.pull_peq()
    }

    /// Pushes a new PEQ configuration to the connected USB DAC device.
    pub fn push_peq(&self, payload: PushPayload) -> oneshot::Receiver<OperationResult> {
        self.worker.push_peq(payload)
    }
}

impl Default for HardwareService {
    fn default() -> Self {
        Self::new()
    }
}
