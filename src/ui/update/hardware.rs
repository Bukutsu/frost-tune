// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::{OperationResult, PushPayload};
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::error::ErrorKind;
use crate::ui::components::connection::ConnectionStatus;
use crate::ui::messages::*;
use crate::ui::state::AppState;
use iced::Task;
use std::sync::Arc;

fn is_hw_busy(window: &AppState) -> bool {
    window.connection.worker.is_none()
        || window.connection.operation_lock.is_pulling
        || window.connection.operation_lock.is_pushing
        || window.connection.operation_lock.is_connecting
}

pub fn handle_hardware(window: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Editor(EditorMessage::PullPressed) => {
            if is_hw_busy(window) {
                return Task::none();
            }

            if window.editor.session.is_dirty {
                window.editor.session.pending_confirm =
                    crate::ui::components::editor::ConfirmAction::PullDevice;
                return Task::none();
            }

            perform_pull(window)
        }
        Message::Editor(EditorMessage::ConfirmPullPressed) => {
            window.editor.session.pending_confirm =
                crate::ui::components::editor::ConfirmAction::None;
            perform_pull(window)
        }
        Message::Editor(EditorMessage::PushPressed) => {
            if is_hw_busy(window) {
                return Task::none();
            }

            if window.editor.session.is_dirty {
                window.editor.session.pending_confirm =
                    crate::ui::components::editor::ConfirmAction::PushToDevice;
                return Task::none();
            }

            perform_push(window)
        }
        Message::Editor(EditorMessage::ConfirmPushPressed) => {
            window.editor.session.pending_confirm =
                crate::ui::components::editor::ConfirmAction::None;
            perform_push(window)
        }
        Message::Editor(EditorMessage::ForceResetPressed) => {
            if is_hw_busy(window) {
                return Task::none();
            }

            window.editor.session.pending_confirm =
                crate::ui::components::editor::ConfirmAction::ForceReset;
            Task::none()
        }
        Message::Editor(EditorMessage::ConfirmForceResetPressed) => {
            window.editor.session.pending_confirm =
                crate::ui::components::editor::ConfirmAction::None;
            perform_force_reset(window)
        }
        Message::Editor(EditorMessage::WorkerPulled(result)) => {
            window.connection.operation_lock.is_pulling = false;
            if result.success {
                window.editor.session.is_dirty = false;
                if let Some(peq) = result.data {
                    let matched = window
                        .editor
                        .ui
                        .profiles
                        .iter()
                        .find(|p| p.data.matches_within(&peq, 0.05, 0.05))
                        .map(|p| p.name.clone());

                    window.editor.data.filters = peq.filters;
                    window.editor.data.global_gain = peq.global_gain;
                    window.editor.data.generation += 1;

                    let status_msg = if let Some(name) = matched {
                        window.editor.ui.selected_profile_name = Some(name.clone());
                        window.editor.ui.eq_source = crate::ui::messages::EqSource::Profile;
                        format!("Device matches profile: {}", name)
                    } else {
                        window.editor.ui.selected_profile_name = None;
                        window.editor.ui.eq_source = crate::ui::messages::EqSource::Pulled;
                        "Data pulled from device".to_string()
                    };

                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Pull successful",
                    ));
                    return window.set_status(status_msg, StatusSeverity::Success);
                }
                Task::none()
            } else if let Some(err) = result.error {
                if err.kind == ErrorKind::OperationCancelled {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Pull operation was interrupted",
                    ));
                    return Task::none();
                }

                let msg = if err.kind == ErrorKind::NotConnected
                    || err.kind == ErrorKind::PolkitAuthRequired
                {
                    window.connection.status =
                        ConnectionStatus::Error("Device lost during operation".into());
                    "Device lost during operation".to_string()
                } else {
                    window.connection.status =
                        ConnectionStatus::Error(err.user_message().to_string());
                    err.user_message().to_string()
                };
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Pull failed: {}", err.message),
                ));
                if let Some(context) = err.context.clone() {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        context,
                    ));
                }
                window.set_status(format!("Pull failed: {}", msg), StatusSeverity::Error)
            } else {
                Task::none()
            }
        }
        Message::Editor(EditorMessage::WorkerPushed(result)) => {
            window.connection.operation_lock.is_pushing = false;
            if result.success {
                window.editor.session.is_dirty = false;
                if let Some(peq) = result.data {
                    window.editor.data.filters = peq.filters;
                    window.editor.data.global_gain = peq.global_gain;
                    window.editor.data.generation += 1;
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Push successful",
                    ));
                    return window
                        .set_status("Settings applied and verified", StatusSeverity::Success);
                }
                Task::none()
            } else if let Some(err) = result.error {
                if err.kind == ErrorKind::OperationCancelled {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Push operation was interrupted",
                    ));
                    return Task::none();
                }

                let base_msg = if err.kind == ErrorKind::NotConnected {
                    window.connection.status =
                        ConnectionStatus::Error("Device lost during operation".into());
                    "Device lost during operation".to_string()
                } else if err.kind == ErrorKind::DeviceLost {
                    window.connection.status =
                        ConnectionStatus::Error("Device disconnected".into());
                    window.connection.disconnect_reason =
                        crate::ui::components::connection::DisconnectReason::DeviceLost;
                    "Device disconnected. Please reconnect and try again.".to_string()
                } else if err.kind == ErrorKind::DeviceBusy {
                    "Device is busy. Close other applications using the device and retry."
                        .to_string()
                } else if err.kind == ErrorKind::ReadTimeout {
                    "Read timeout. Device may be unresponsive. Try reconnecting.".to_string()
                } else if err.kind == ErrorKind::PolkitAuthRequired {
                    "Authentication required to access USB DAC on Linux. Approve the polkit prompt and retry.".to_string()
                } else {
                    window.connection.status =
                        ConnectionStatus::Error(err.user_message().to_string());
                    err.user_message().to_string()
                };

                let full_msg = format!(
                    "Push failed: {}. Try reading from device to resync.",
                    base_msg
                );

                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Push failed: {}", err.message),
                ));
                if let Some(context) = err.context.clone() {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        context,
                    ));
                }
                window.set_status(full_msg, StatusSeverity::Error)
            } else {
                Task::none()
            }
        }
        _ => Task::none(),
    }
}

fn unwrap_operation_result(result: Result<OperationResult, String>) -> OperationResult {
    match result {
        Ok(res) => res,
        Err(e) => OperationResult {
            success: false,
            data: None,
            error: Some(crate::error::AppError::new(
                crate::error::ErrorKind::IpcError,
                e,
            )),
        },
    }
}

fn perform_pull(window: &mut AppState) -> Task<Message> {
    window.connection.operation_lock.is_pulling = true;
    window.diagnostics.push(DiagnosticEvent::new(
        LogLevel::Info,
        Source::UI,
        "Pulling from device",
    ));
    let worker = match window.connection.worker.as_ref() {
        Some(w) => Arc::clone(w),
        None => return Task::none(),
    };
    let pull_task = Task::perform(
        async move {
            let result = worker.pull_peq().await;
            unwrap_operation_result(result)
        },
        |res| Message::Editor(EditorMessage::WorkerPulled(res)),
    );
    let status_task = window.set_status("Reading from device...", StatusSeverity::Info);
    Task::batch(vec![pull_task, status_task])
}

fn perform_push(window: &mut AppState) -> Task<Message> {
    window.connection.operation_lock.is_pushing = true;
    window.diagnostics.push(DiagnosticEvent::new(
        LogLevel::Info,
        Source::UI,
        "Push pressed",
    ));
    let worker = match window.connection.worker.as_ref() {
        Some(w) => Arc::clone(w),
        None => return Task::none(),
    };
    let filters = window.editor.data.filters.clone();
    let global_gain = window.editor.data.global_gain;
    let push_task = Task::perform(
        async move {
            let payload = PushPayload {
                filters,
                global_gain: Some(global_gain),
            };
            let result = worker.push_peq(payload).await;
            unwrap_operation_result(result)
        },
        |res| Message::Editor(EditorMessage::WorkerPushed(res)),
    );
    let status_task = window.set_status("Writing to device...", StatusSeverity::Info);
    Task::batch(vec![push_task, status_task])
}

fn perform_force_reset(window: &mut AppState) -> Task<Message> {
    window.connection.operation_lock.is_pushing = true;
    window.diagnostics.push(DiagnosticEvent::new(
        LogLevel::Warn,
        Source::UI,
        "Force Reset triggered",
    ));
    let worker = match window.connection.worker.as_ref() {
        Some(w) => Arc::clone(w),
        None => return Task::none(),
    };
    let push_task = Task::perform(
        async move {
            let result = worker.reset_peq().await;
            unwrap_operation_result(result)
        },
        |res| Message::Editor(EditorMessage::WorkerPushed(res)),
    );
    let status_task = window.set_status("Resetting device to flat...", StatusSeverity::Warning);
    Task::batch(vec![push_task, status_task])
}
