use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::{BackendKind, WorkerStatus};
use crate::models::{ConnectionResult, Device, OperationResult};
use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::{ConfirmAction, ConnectionStatus, DisconnectReason, MainWindow};
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
            rx.recv_timeout(std::time::Duration::from_secs(2))
                .unwrap_or(WorkerStatus {
                    connected: false,
                    physically_present: false,
                    device: None,
                    available_devices: Vec::new(),
                    backend_reset: false,
                    generation: 0,
                    fatal_error: None,
                })
        },
        Message::WorkerStatus,
    )
}

fn maybe_reconnect(window: &mut MainWindow) -> Option<Task<Message>> {
    if window.disconnect_reason == DisconnectReason::DeviceLost
        && !window.operation_lock.is_connecting
        && window.connection_status != ConnectionStatus::Connected
        && !window.available_devices.is_empty()
    {
        let should_attempt = match window.last_auto_reconnect_attempt {
            None => true,
            Some(last) => {
                let backoff_secs = (2u64.saturating_pow(window.auto_reconnect_attempts)).min(30);
                std::time::Instant::now().duration_since(last)
                    >= std::time::Duration::from_secs(backoff_secs)
            }
        };
        if should_attempt {
            window.last_auto_reconnect_attempt = Some(std::time::Instant::now());
            window.auto_reconnect_attempts += 1;
            window.connection_status = ConnectionStatus::Connecting;
            window.operation_lock.is_connecting = true;
            window.suspend_status_polling = true;
            let target_device = window.available_devices.first().cloned().unwrap();
            let worker = match window.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return None,
            };
            Some(Task::perform(
                async move {
                    let rx = worker.connect(Some(target_device), Some(BackendKind::Local));
                    rx.recv_timeout(std::time::Duration::from_secs(5))
                        .unwrap_or_else(|_| timed_out_connection_result("Auto-reconnect timed out"))
                },
                Message::WorkerConnected,
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn maybe_check_profiles(window: &mut MainWindow) -> Option<Task<Message>> {
    let profile_check_interval = if window.connection_status == ConnectionStatus::Connected {
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
            Message::ProfilesDirMtimeChecked,
        ))
    } else {
        None
    }
}

pub fn handle_connection(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::ClearStatusMessage(id) => {
            if let Some(ref status) = window.editor_state.session.status_message {
                if status.id == id {
                    window.editor_state.session.status_message = None;
                }
            }
            Task::none()
        }
        Message::DismissConfirmDialog => {
            window.editor_state.session.pending_confirm = ConfirmAction::None;
            window.editor_state.session.import_name_input = String::new();
            Task::none()
        }
        Message::WindowCloseRequested(id) => {
            if window.editor_state.session.is_dirty {
                window.editor_state.session.pending_confirm =
                    ConfirmAction::ExitWithUnsavedChanges(id);
                Task::none()
            } else {
                iced::window::close(id)
            }
        }
        Message::ConfirmExit(id) => {
            window.editor_state.session.pending_confirm = ConfirmAction::None;
            iced::window::close(id)
        }

        Message::SaveAndExit(id) => {
            let save_name = if !window
                .editor_state
                .session
                .new_profile_name
                .trim()
                .is_empty()
            {
                Some(window.editor_state.session.new_profile_name.clone())
            } else {
                window.editor_state.ui.selected_profile_name.clone()
            };

            if let Some(name) = save_name {
                let peq_data = crate::models::PEQData {
                    filters: window.editor_state.data.filters.clone(),
                    global_gain: window.editor_state.data.global_gain,
                };
                match crate::storage::save_profile(&name, &peq_data) {
                    Ok(()) => {
                        window.editor_state.session.is_dirty = false;
                        window.editor_state.session.pending_confirm = ConfirmAction::None;
                        return iced::window::close(id);
                    }
                    Err(e) => {
                        window.editor_state.session.pending_confirm = ConfirmAction::None;
                        return window
                            .set_status(format!("Save failed: {}", e), StatusSeverity::Error);
                    }
                }
            }
            window.set_status(
                "Enter a profile name first, then try Save & Exit again.",
                StatusSeverity::Warning,
            )
        }
        Message::ConnectPressed(device) => {
            if window.worker.is_none() {
                return Task::none();
            }
            window.connection_status = ConnectionStatus::Connecting;
            window.operation_lock.is_connecting = true;
            window.suspend_status_polling = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Connect pressed for {}", device.path),
            ));
            let worker = match window.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let connect_task = Task::perform(
                async move {
                    let backend = Some(BackendKind::Local);

                    let rx = worker.connect(Some(device), backend);
                    rx.recv_timeout(std::time::Duration::from_secs(5))
                        .unwrap_or_else(|_| {
                            timed_out_connection_result("Connection request timed out")
                        })
                },
                Message::WorkerConnected,
            );
            let status_task = window.set_status("Connecting to device...", StatusSeverity::Info);
            Task::batch(vec![connect_task, status_task])
        }
        Message::DisconnectPressed => {
            if window.worker.is_none() {
                return Task::none();
            }
            window.disconnect_reason = DisconnectReason::Manual;
            window.operation_lock.is_disconnecting = true;
            window.suspend_status_polling = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                "Disconnect pressed",
            ));
            let worker = match window.worker.as_ref() {
                Some(w) => Arc::clone(w),
                None => return Task::none(),
            };
            let disconnect_task = Task::perform(
                async move {
                    let rx = worker.disconnect();
                    rx.recv_timeout(std::time::Duration::from_secs(5))
                        .unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(
                                ErrorKind::IpcError,
                                "Disconnect request timed out",
                            )),
                        })
                },
                Message::WorkerDisconnected,
            );
            let status_task = window.set_status("Disconnecting...", StatusSeverity::Info);
            Task::batch(vec![disconnect_task, status_task])
        }
        Message::WorkerConnected(result) => {
            window.operation_lock.is_connecting = false;
            window.suspend_status_polling = false;
            let device_name_owned = if let Some(ref d) = result.device {
                Device::from_vid_pid(d.vendor_id, d.product_id)
                    .name()
                    .to_string()
            } else {
                "Unknown Device".to_string()
            };

            if result.success {
                window.connection_status = ConnectionStatus::Connected;
                window.connected_device = result.device;
                window.disconnect_reason = DisconnectReason::None;
                window.last_auto_reconnect_attempt = None;
                window.auto_reconnect_attempts = 0;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    format!("Connected to {}", device_name_owned),
                ));
                window.set_status(
                    format!("Connected to {}", device_name_owned),
                    StatusSeverity::Success,
                )
            } else {
                let err = result
                    .error
                    .unwrap_or_else(|| AppError::new(ErrorKind::Unknown, "Unknown error"));
                let user_error = match err.kind {
                    ErrorKind::PolkitAuthRequired => err.message.clone(),
                    _ => err.user_message().to_string(),
                };
                window.connection_status = ConnectionStatus::Error(user_error.clone());
                window.connected_device = None;
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
        Message::WorkerDisconnected(_) => {
            window.operation_lock.is_disconnecting = false;
            window.suspend_status_polling = false;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::Worker,
                "Disconnected",
            ));
            window.connection_status = ConnectionStatus::Disconnected;
            window.connected_device = None;
            window.editor_state.session.pending_confirm = ConfirmAction::None;
            if window.disconnect_reason == DisconnectReason::Manual {
                window.last_auto_reconnect_attempt = None;
                window.auto_reconnect_attempts = 0;
            }
            window.set_status("Disconnected", StatusSeverity::Info)
        }
        Message::WorkerStatus(status) => {
            if status.backend_reset {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Warn,
                    Source::Worker,
                    "Worker backend reset",
                ));
                return Task::perform(async {}, |_| Message::WorkerBackendReset);
            }
            if status.generation < window.connection_generation {
                return Task::none();
            }
            window.connection_generation = status.generation;
            if window.available_devices != status.available_devices {
                window.available_devices = status.available_devices.clone();
            }

            // Ignore contradictory status updates during manual transition
            if window.operation_lock.is_connecting || window.operation_lock.is_disconnecting {
                return Task::none();
            }

            window.connected_device = if status.connected {
                status.device.clone()
            } else {
                None
            };

            if status.connected && window.connection_status != ConnectionStatus::Connected {
                window.connection_status = ConnectionStatus::Connected;
                window.disconnect_reason = DisconnectReason::None;
                window.last_auto_reconnect_attempt = None;
                window.auto_reconnect_attempts = 0;
                log::info!("Device connected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device connected (poll)",
                ));
            } else if !status.connected && window.connection_status == ConnectionStatus::Connected {
                window.connection_status = ConnectionStatus::Disconnected;
                window.disconnect_reason = DisconnectReason::DeviceLost;
                log::info!("Device disconnected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device disconnected (poll)",
                ));
            }

            if let Some(ref fatal) = status.fatal_error {
                if !matches!(window.connection_status, ConnectionStatus::Error(_)) {
                    window.connection_status = ConnectionStatus::Error(fatal.clone());
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
            Task::none()
        }
        Message::Tick(_) => {
            let worker = match &window.worker {
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
            if !window.suspend_status_polling {
                tasks.push(status_task);
            }
            if let Some(task) = reconnect_task {
                tasks.push(task);
            }
            Task::batch(tasks)
        }
        Message::WorkerBackendReset => {
            window.connection_status = ConnectionStatus::Error("Worker backend reset".into());
            window.connected_device = None;
            window.operation_lock.is_connecting = false;
            window.operation_lock.is_pulling = false;
            window.operation_lock.is_pushing = false;
            window.operation_lock.is_disconnecting = false;
            window.suspend_status_polling = false;
            window.set_status("Connection lost. Please reconnect.", StatusSeverity::Error)
        }
        Message::ProfilesDirMtimeChecked(mtime) => {
            if mtime != window.editor_state.ui.profiles_dir_mtime {
                let reload_task = Task::perform(
                    async move { crate::storage::load_all_profiles() },
                    Message::ProfilesLoaded,
                );
                return reload_task;
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
