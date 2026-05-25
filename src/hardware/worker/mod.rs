// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

mod state;

use tokio::sync::{mpsc, oneshot};

use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
pub use backend::BackendKind;

pub mod backend;
pub mod connection;
pub mod ops;

#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub connected: bool,
    pub physically_present: bool,
    pub device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
    pub backend_reset: bool,
    pub generation: u64,
    pub fatal_error: Option<String>,
}

pub enum UsbCommand {
    Connect(
        Option<DeviceInfo>,
        Option<BackendKind>,
        oneshot::Sender<ConnectionResult>,
    ),
    Disconnect(oneshot::Sender<OperationResult>),
    Status(oneshot::Sender<WorkerStatus>),
    PullPEQ(oneshot::Sender<OperationResult>),
    PushPEQ(PushPayload, oneshot::Sender<OperationResult>),
}

pub struct UsbWorker {
    tx: mpsc::Sender<UsbCommand>,
}

impl UsbWorker {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(32);

        // We use a dedicated OS thread because hidapi is not Send.
        // This thread runs a local Tokio runtime to handle the async IPC calls.
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build local runtime for UsbWorker");

            rt.block_on(async {
                let mut state = state::WorkerState::new();
                loop {
                    match state.run_iteration(&mut rx).await {
                        state::IterationResult::Continue => {}
                        state::IterationResult::Stop => break,
                    }
                }
            });
        });

        UsbWorker { tx }
    }

    pub fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> oneshot::Receiver<ConnectionResult> {
        let (tx, rx) = oneshot::channel();
        let cmd_tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = cmd_tx.send(UsbCommand::Connect(device, backend, tx)).await;
        });
        rx
    }

    pub fn disconnect(&self) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let cmd_tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = cmd_tx.send(UsbCommand::Disconnect(tx)).await;
        });
        rx
    }

    pub fn status(&self) -> oneshot::Receiver<WorkerStatus> {
        let (tx, rx) = oneshot::channel();
        let cmd_tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = cmd_tx.send(UsbCommand::Status(tx)).await;
        });
        rx
    }

    pub fn pull_peq(&self) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let cmd_tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = cmd_tx.send(UsbCommand::PullPEQ(tx)).await;
        });
        rx
    }

    pub fn push_peq(&self, payload: PushPayload) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let cmd_tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = cmd_tx.send(UsbCommand::PushPEQ(payload, tx)).await;
        });
        rx
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
    }
}
