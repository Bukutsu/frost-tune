// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

mod state;

use std::sync::mpsc;
use std::thread;
use tokio::sync::oneshot;

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

fn panic_message(panic_info: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic in worker thread".to_string()
    }
}

impl UsbWorker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut state = state::WorkerState::new();

            loop {
                let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    state.run_iteration(&rx)
                }));

                match panic_result {
                    Ok(state::IterationResult::Continue) => {}
                    Ok(state::IterationResult::Stop) => break,
                    Err(panic_info) => {
                        let msg = panic_message(&panic_info);
                        log::error!("Worker thread panicked: {}", msg);
                        state.fatal_error = Some(msg);
                        crate::hardware::hid::reset_nonce();
                        state.backend = None;
                        state.generation = state.generation.saturating_add(1);
                    }
                }
            }
        });

        UsbWorker { tx }
    }

    pub fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> oneshot::Receiver<ConnectionResult> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(UsbCommand::Connect(device, backend, tx));
        rx
    }

    pub fn disconnect(&self) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(UsbCommand::Disconnect(tx));
        rx
    }

    pub fn status(&self) -> oneshot::Receiver<WorkerStatus> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(UsbCommand::Status(tx));
        rx
    }

    pub fn pull_peq(&self) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(UsbCommand::PullPEQ(tx));
        rx
    }

    pub fn push_peq(&self, payload: PushPayload) -> oneshot::Receiver<OperationResult> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(UsbCommand::PushPEQ(payload, tx));
        rx
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
    }
}
