// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::{BackendKind, WorkerStatus};
use crate::hardware::{ConnectionResult, OperationResult};
use crate::ui::components::connection::{ConnectionStatus, DisconnectReason};
use crate::ui::components::editor::ConfirmAction;
use crate::ui::messages::*;
use crate::ui::state::AppState;
use iced::Task;
use std::sync::Arc;

fn timed_out_connection_result(error_message: &str) -> ConnectionResult {
    ConnectionResult {
        success: false,
        device: None,
        error: Some(AppError::new(ErrorKind::IpcError, error_message)),
    }
}

fn poll_worker_status(worker: Arc<crate::hardware::worker::UsbWorker>) -> Task<Message> {
    Task::perform(
        async move {
            let rx = worker.status();
            match tokio::time::timeout(std::time::Duration::from_secs(2), rx).await {
                Ok(Ok(status)) => status,
                _ => WorkerStatus {
                    connected: false,
                    physically_present: false,
                    device: None,
                    available_devices: Vec::new(),
                    backend_reset: false,
                    generation: 0,
                    fatal_error: None,
                },
            }
        },
        |status| Message::Connection(ConnectionMessage::WorkerStatus(status)),
    )
}

fn maybe_reconnect(window: &mut AppState) -> Option<Task<Message>> {
    if window.connection.disconnect_reason == DisconnectReason::DeviceLost
        && !window.connection.operation_lock.is_connecting
        && window.connection.status != ConnectionStatus::Connected
        && !window.connection.available_devices.is_empty()
    {
        let should_attempt = match window.connection.last_auto_reconnect_attempt {
            None => true,
            Some(last) => {
                let backoff_secs =
                    (2u64.saturating_pow(window.connection.auto_reconnect_attempts)).min(30);
                std::time::Instant::now().duration_since(last)
                    >= std::time::Duration::from_secs(backoff_secs)
            }
        };
        if should_attempt {
            let target_device = window.connection.available_devices.first().cloned()?;

            window.connection.last_auto_reconnect_attempt = Some(std::time::Instant::now());
            window.connection.auto_reconnect_attempts += 1;
            window.connection.status = ConnectionStatus::Connecting;
            window.connection.operation_lock.is_connecting = true;
            window.connection.suspend_status_polling = true;

            let worker = match window.connection.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return None,
            };
            Some(Task::perform(
                async move {
                    let rx = worker.connect(Some(target_device), Some(BackendKind::Local));
                    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
                        Ok(Ok(res)) => res,
                        _ => timed_out_connection_result("Auto-reconnect timed out"),
                    }
                },
                |res| Message::Connection(ConnectionMessage::WorkerConnected(res)),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn maybe_check_profiles(window: &mut AppState) -> Option<Task<Message>> {
    let profile_check_interval = if window.connection.status == ConnectionStatus::Connected {
        std::time::Duration::from_secs(10)
    } else {
        std::time::Duration::from_secs(30)
    };

    let should_check_profiles = match window.last_profile_check {
        None => true,
        Some(last) => std::time::Instant::now().duration_since(last) >= profile_check_interval,
    };

    if should_check_profiles {
        window.last_profile_check = Some(std::time::Instant::now());
        Some(Task::perform(
            async move { crate::storage::get_profiles_dir_mtime() },
            |mtime| Message::Profiles(ProfilesMessage::ProfilesDirMtimeChecked(mtime)),
        ))
    } else {
        None
    }
}

pub fn handle_connection(window: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::ClearStatusMessage(id) => {
            if let Some(ref status) = window.editor.session.status_message {
                if status.id == id {
                    window.editor.session.status_message = None;
                }
            }
            Task::none()
        }
        Message::DismissConfirmDialog => {
            window.editor.session.pending_confirm = ConfirmAction::None;
            window.editor.session.import_name_input = String::new();
            Task::none()
        }
        Message::CloseRequested(id) => {
            if window.editor.session.is_dirty {
                window.editor.session.pending_confirm = ConfirmAction::ExitWithUnsavedChanges(id);
                Task::none()
            } else {
                iced::window::close(id)
            }
        }
        Message::ConfirmExit(id) => {
            window.editor.session.pending_confirm = ConfirmAction::None;
            iced::window::close(id)
        }

        Message::SaveAndExit(id) => {
            let save_name = if !window.editor.session.new_profile_name.trim().is_empty() {
                Some(window.editor.session.new_profile_name.clone())
            } else {
                window.editor.ui.selected_profile_name.clone()
            };

            if let Some(name) = save_name {
                let peq_data = std::sync::Arc::new(crate::core::PEQData {
                    filters: window.editor.data.filters.clone(),
                    global_gain: window.editor.data.global_gain,
                });
                let name_clone = name.clone();
                let peq_data_clone = peq_data.clone();

                Task::perform(
                    async move {
                        crate::storage::save_profile(&name_clone, &peq_data_clone)
                            .await
                            .map_err(|e| {
                                crate::error::AppError::new(
                                    crate::error::ErrorKind::StorageError,
                                    e.to_string(),
                                )
                            })
                    },
                    move |result| {
                        Message::Profiles(ProfilesMessage::ProfileSaved {
                            name: name.clone(),
                            data: peq_data.clone(),
                            result,
                            context: crate::ui::messages::SaveContext::Exit(id),
                        })
                    },
                )
            } else {
                window.set_status(
                    "Enter a profile name first, then try Save & Exit again.",
                    StatusSeverity::Warning,
                )
            }
        }
        Message::Connection(ConnectionMessage::ConnectPressed(device)) => {
            if window.connection.worker.is_none() {
                return Task::none();
            }
            window.connection.status = ConnectionStatus::Connecting;
            window.connection.operation_lock.is_connecting = true;
            window.connection.suspend_status_polling = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Connect pressed for {}", device.path),
            ));
            let worker = match window.connection.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let connect_task = Task::perform(
                async move {
                    let backend = Some(BackendKind::Local);

                    let rx = worker.connect(Some(device), backend);
                    match tokio::time::timeout(std::time::Duration::from_secs(60), rx).await {
                        Ok(Ok(res)) => res,
                        _ => timed_out_connection_result("Connection request timed out"),
                    }
                },
                |res| Message::Connection(ConnectionMessage::WorkerConnected(res)),
            );
            connect_task
        }
        Message::Connection(ConnectionMessage::DisconnectPressed) => {
            if window.connection.worker.is_none() {
                return Task::none();
            }
            window.connection.disconnect_reason = DisconnectReason::Manual;
            window.connection.operation_lock.is_disconnecting = true;
            window.connection.suspend_status_polling = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                "Disconnect pressed",
            ));
            let worker = match window.connection.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let disconnect_task = Task::perform(
                async move {
                    let rx = worker.disconnect();
                    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
                        Ok(Ok(res)) => res,
                        _ => OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(
                                ErrorKind::IpcError,
                                "Disconnect request timed out",
                            )),
                        },
                    }
                },
                |res| Message::Connection(ConnectionMessage::WorkerDisconnected(res)),
            );
            disconnect_task
        }
        Message::Connection(ConnectionMessage::WorkerConnected(result)) => {
            window.connection.operation_lock.is_connecting = false;
            window.connection.suspend_status_polling = false;
            let device_name_owned = if let Some(ref d) = result.device {
                crate::hardware::get_profile(d.vendor_id, d.product_id)
                    .map(|p| p.name())
                    .unwrap_or("Unknown Device")
                    .to_string()
            } else {
                "Unknown Device".to_string()
            };

            if result.success {
                window.connection.status = ConnectionStatus::Connected;
                window.connection.connected_device = result.device;
                window.connection.disconnect_reason = DisconnectReason::None;
                window.connection.last_auto_reconnect_attempt = None;
                window.connection.auto_reconnect_attempts = 0;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    format!("Connected to {}", device_name_owned),
                ));
                if window.editor.ui.auto_pull_on_connect {
                    Task::done(Message::Editor(EditorMessage::PullPressed))
                } else {
                    Task::none()
                }
            } else {
                let err = result
                    .error
                    .unwrap_or_else(|| AppError::new(ErrorKind::Unknown, "Unknown error"));
                let user_error = match err.kind {
                    ErrorKind::PolkitAuthRequired => err.message.clone(),
                    _ => err.user_message().to_string(),
                };
                window.connection.status = ConnectionStatus::Error(user_error.clone());
                window.connection.connected_device = None;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Connect failed: {}", err.message),
                ));
                if let Some(context) = err.context.clone() {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        context,
                    ));
                }
                window.set_status(
                    format!("Connect failed: {}", user_error),
                    StatusSeverity::Error,
                )
            }
        }
        Message::Connection(ConnectionMessage::WorkerDisconnected(_)) => {
            window.connection.operation_lock.is_disconnecting = false;
            window.connection.suspend_status_polling = false;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::Worker,
                "Disconnected",
            ));
            window.connection.status = ConnectionStatus::Disconnected;
            window.connection.connected_device = None;
            window.editor.session.pending_confirm = ConfirmAction::None;
            if window.connection.disconnect_reason == DisconnectReason::Manual {
                window.connection.last_auto_reconnect_attempt = None;
                window.connection.auto_reconnect_attempts = 0;
            }
            Task::none()
        }
        Message::Connection(ConnectionMessage::WorkerStatus(status)) => {
            if status.backend_reset {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Warn,
                    Source::Worker,
                    "Worker backend reset",
                ));
                return Task::perform(async {}, |_| {
                    Message::Connection(ConnectionMessage::WorkerBackendReset)
                });
            }
            if status.generation < window.connection.connection_generation {
                return Task::none();
            }
            window.connection.connection_generation = status.generation;
            if window.connection.available_devices != status.available_devices {
                window.connection.available_devices = status.available_devices.clone();
            }

            // Ignore contradictory status updates during manual transition
            if window.connection.operation_lock.is_connecting
                || window.connection.operation_lock.is_disconnecting
            {
                return Task::none();
            }

            window.connection.connected_device = if status.connected {
                status.device.clone()
            } else {
                None
            };

            let mut on_connect_task: Task<Message> = Task::none();
            if status.connected && window.connection.status != ConnectionStatus::Connected {
                window.connection.status = ConnectionStatus::Connected;
                window.connection.disconnect_reason = DisconnectReason::None;
                window.connection.last_auto_reconnect_attempt = None;
                window.connection.auto_reconnect_attempts = 0;
                log::info!("Device connected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device connected (poll)",
                ));
                if window.editor.ui.auto_pull_on_connect {
                    on_connect_task = Task::done(Message::Editor(EditorMessage::PullPressed));
                }
            } else if !status.connected && window.connection.status == ConnectionStatus::Connected {
                window.connection.status = ConnectionStatus::Disconnected;
                window.connection.disconnect_reason = DisconnectReason::DeviceLost;
                log::info!("Device disconnected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device disconnected (poll)",
                ));
            }

            if let Some(ref fatal) = status.fatal_error {
                if !matches!(window.connection.status, ConnectionStatus::Error(_)) {
                    window.connection.status = ConnectionStatus::Error(fatal.clone());
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Worker fatal error: {}", fatal),
                    ));
                    return window.set_status(
                        format!("Hardware access error: {}", fatal),
                        StatusSeverity::Error,
                    );
                }
            }
            on_connect_task
        }
        Message::Tick(_) => {
            let worker = match &window.connection.worker {
                Some(w) => w,
                None => return Task::none(),
            };
            let worker = Arc::clone(worker);
            let status_task = poll_worker_status(worker);
            let reconnect_task = maybe_reconnect(window);
            let profile_task = maybe_check_profiles(window);

            let mut tasks: Vec<Task<Message>> = Vec::new();

            if let Some(task) = profile_task {
                tasks.push(task);
            }
            if !window.connection.suspend_status_polling {
                tasks.push(status_task);
            }
            if let Some(task) = reconnect_task {
                tasks.push(task);
            }
            Task::batch(tasks)
        }
        Message::Connection(ConnectionMessage::WorkerBackendReset) => {
            window.connection.status = ConnectionStatus::Error("Worker backend reset".into());
            window.connection.connected_device = None;
            window.connection.operation_lock.is_connecting = false;
            window.connection.operation_lock.is_pulling = false;
            window.connection.operation_lock.is_pushing = false;
            window.connection.operation_lock.is_disconnecting = false;
            window.connection.suspend_status_polling = false;
            window.set_status("Connection lost. Please reconnect.", StatusSeverity::Error)
        }
        Message::Profiles(ProfilesMessage::ProfilesDirMtimeChecked(mtime)) => {
            if mtime != window.editor.ui.profiles_dir_mtime {
                let reload_task = Task::perform(
                    async move { crate::storage::load_all_profiles().await },
                    |res| Message::Profiles(ProfilesMessage::ProfilesLoaded(res)),
                );
                return reload_task;
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
