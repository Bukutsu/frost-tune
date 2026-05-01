use crate::ui::state::{MainWindow, ConnectionStatus};
use crate::ui::messages::{Message, StatusSeverity};
use crate::error::{AppError, ErrorKind};
use crate::models::{OperationResult, PushPayload};
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use iced::Task;
use std::sync::Arc;

pub fn handle_hardware(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::PullPressed => {
            if window.worker.is_none()
                || window.operation_lock.is_pulling
                || window.operation_lock.is_pushing
                || window.operation_lock.is_connecting
            {
                return Task::none();
            }
            window.operation_lock.is_pulling = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                "Pull pressed",
            ));
            let worker = match window.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let pull_task = Task::perform(
                async move {
                    let rx = worker.pull_peq();
                    match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                        Ok(res) => res,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Operation timed out after 30 seconds")),
                        },
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Background worker unexpectedly terminated")),
                        },
                    }
                },
                Message::WorkerPulled,
            );
            let status_task = window.set_status("Reading from device...", StatusSeverity::Info);
            Task::batch(vec![pull_task, status_task])
        }
        Message::PushPressed => {
            if window.worker.is_none()
                || window.operation_lock.is_pulling
                || window.operation_lock.is_pushing
                || window.operation_lock.is_connecting
            {
                return Task::none();
            }
            window.operation_lock.is_pushing = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                "Push pressed",
            ));
            let worker = match window.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let filters = window.editor_state.filters.clone();
            let global_gain = window.editor_state.global_gain;
            let push_task = Task::perform(
                async move {
                    let payload = PushPayload {
                        filters,
                        global_gain: Some(global_gain),
                    };
                    let rx = worker.push_peq(payload);
                    match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                        Ok(res) => res,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Operation timed out after 30 seconds")),
                        },
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Background worker unexpectedly terminated")),
                        },
                    }
                },
                Message::WorkerPushed,
            );
            let status_task = window.set_status("Writing to device...", StatusSeverity::Info);
            Task::batch(vec![push_task, status_task])
        }
        Message::WorkerPulled(result) => {
            window.operation_lock.is_pulling = false;
            if result.success {
                if let Some(peq) = result.data {
                    window.editor_state.filters = peq
                        .filters
                        .into_iter()
                        .map(|mut f| {
                            f.enabled = true;
                            f
                        })
                        .collect();
                    window.editor_state.global_gain = peq.global_gain;
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Pull successful",
                    ));
                    return window
                        .set_status("Data pulled from device", StatusSeverity::Success);
                }
                Task::none()
            } else if let Some(err) = result.error {
                if err.kind == ErrorKind::NotConnected || err.kind == ErrorKind::PolkitAuthRequired {
                    window.connection_status = ConnectionStatus::Disconnected;
                } else {
                    window.connection_status = ConnectionStatus::Error(err.message.clone());
                }
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Connection failed: {}", err.message),
                ));
                window.set_status(format!("Connection failed: {}", err.message), StatusSeverity::Error)
            } else {
                Task::none()
            }
        }
        Message::WorkerPushed(result) => {
            window.operation_lock.is_pushing = false;
            if result.success {
                if let Some(peq) = result.data {
                    window.editor_state.filters = peq
                        .filters
                        .into_iter()
                        .map(|mut f| {
                            f.enabled = true;
                            f
                        })
                        .collect();
                    window.editor_state.global_gain = peq.global_gain;
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Push successful",
                    ));
                    return window.set_status(
                        "Settings applied and verified",
                        StatusSeverity::Success,
                    );
                }
                Task::none()
            } else if let Some(err) = result.error {
                if err.kind == ErrorKind::NotConnected {
                    window.connection_status = ConnectionStatus::Disconnected;
                } else {
                    window.connection_status = ConnectionStatus::Error(err.message.clone());
                }
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Push failed: {}", err.message),
                ));
                window.set_status(format!("Push failed: {}", err.message), StatusSeverity::Error)
            } else {
                Task::none()
            }
        }
        _ => Task::none(),
    }
}
