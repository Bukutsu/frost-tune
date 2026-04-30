use crate::ui::state::{MainWindow, ConfirmAction, ConnectionStatus, DisconnectReason};
use crate::ui::messages::{Message, StatusSeverity};
use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::{WorkerStatus, BackendKind};
use crate::models::{ConnectionResult, Device, OperationResult};
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use iced::Task;
use std::sync::Arc;

pub fn handle_connection(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::ClearStatusMessage => {
            window.editor_state.status_message = None;
            Task::none()
        }
        Message::DismissConfirmDialog => {
            window.editor_state.pending_confirm = ConfirmAction::None;
            Task::none()
        }
        Message::DeviceSelected(index) => {
            window.selected_device_index = Some(index);
            Task::none()
        }
        Message::ConnectPressed(device) => {
            if window.worker.is_none() {
                return Task::none();
            }
            window.connection_status = ConnectionStatus::Connecting;
            window.operation_lock.is_connecting = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Connect pressed for {}", device.path),
            ));
            let worker = Arc::clone(window.worker.as_ref().unwrap());
            Task::perform(
                async move {
                    let rx = worker.connect(Some(device), Some(BackendKind::Local));
                    rx.recv().unwrap_or_else(|_| ConnectionResult {
                        success: false,
                        device: None,
                        error: Some(AppError::new(ErrorKind::NotConnected, "Channel closed")),
                    })
                },
                Message::WorkerConnected,
            )
        }
        Message::ConfirmElevatedConnect(device) => {
            window.editor_state.pending_confirm = ConfirmAction::None;
            if window.worker.is_none() {
                return Task::none();
            }
            window.connection_status = ConnectionStatus::Connecting;
            window.operation_lock.is_connecting = true;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Elevated connect confirmed for {}", device.path),
            ));
            let worker = Arc::clone(window.worker.as_ref().unwrap());
            let connect_task = Task::perform(
                async move {
                    #[cfg(target_os = "linux")]
                    let backend = Some(BackendKind::Elevated);
                    #[cfg(not(target_os = "linux"))]
                    let backend = None;
                    
                    let rx = worker.connect(Some(device), backend);
                    rx.recv().unwrap_or(ConnectionResult {
                        success: false,
                        device: None,
                        error: Some("Worker closed".into()),
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
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                "Disconnect pressed",
            ));
            let worker = Arc::clone(window.worker.as_ref().unwrap());
            let disconnect_task = Task::perform(
                async move {
                    let rx = worker.disconnect();
                    rx.recv().unwrap_or(OperationResult {
                        success: false,
                        data: None,
                        error: Some(AppError::new(ErrorKind::Unknown, "Worker closed")),
                    })
                },
                Message::WorkerDisconnected,
            );
            let status_task = window.set_status("Disconnecting...", StatusSeverity::Info);
            Task::batch(vec![disconnect_task, status_task])
        }
        Message::WorkerConnected(result) => {
            window.operation_lock.is_connecting = false;
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
                let err = result.error.unwrap_or_else(|| AppError::new(ErrorKind::Unknown, "Unknown error"));
                if err.kind == ErrorKind::PolkitAuthRequired || err.kind == ErrorKind::PermissionDenied {
                    if let Some(device) = result.device {
                        window.editor_state.pending_confirm = ConfirmAction::ElevatedConnect(device);
                        window.connection_status = ConnectionStatus::Disconnected;
                        return Task::none();
                    }
                }
                let user_error = match err.kind {
                    ErrorKind::PolkitAuthRequired => "Authentication required to access USB DAC on Linux. Approve the polkit prompt and retry.".to_string(),
                    _ => err.message.clone(),
                };
                window.connection_status = ConnectionStatus::Error(user_error.clone());
                window.connected_device = None;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::Worker,
                    format!("Connect failed: {}", err.message),
                ));
                window.set_status(
                    format!("Connect failed: {}", user_error),
                    StatusSeverity::Error,
                )
            }
        }
        Message::WorkerDisconnected(_) => {
            window.operation_lock.is_disconnecting = false;
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::Worker,
                "Disconnected",
            ));
            window.connection_status = ConnectionStatus::Disconnected;
            window.connected_device = None;
            window.set_status("Disconnected", StatusSeverity::Info)
        }
        Message::WorkerStatus(status) => {
            window.available_devices = status.available_devices.clone();
            if let Some(idx) = window.selected_device_index {
                if idx >= window.available_devices.len() {
                    window.selected_device_index = None;
                }
            }
            window.connected_device = if status.connected {
                status.device.clone()
            } else {
                None
            };

            if status.connected && window.connection_status != ConnectionStatus::Connected {
                window.connection_status = ConnectionStatus::Connected;
                window.disconnect_reason = DisconnectReason::None;
                log::info!("Device connected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device connected (poll)",
                ));
            } else if !status.connected && window.connection_status == ConnectionStatus::Connected
            {
                window.connection_status = ConnectionStatus::Disconnected;
                window.disconnect_reason = DisconnectReason::DeviceLost;
                log::info!("Device disconnected");
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Device disconnected (poll)",
                ));
            }
            Task::none()
        }
        Message::Tick(_) => {
            let worker = match &window.worker {
                Some(w) => w,
                None => return Task::none(),
            };
            let worker = Arc::clone(worker);
            Task::perform(
                async move {
                    let rx = worker.status();
                    rx.recv().unwrap_or(WorkerStatus {
                        connected: false,
                        physically_present: false,
                        device: None,
                        available_devices: Vec::new(),
                    })
                },
                Message::WorkerStatus,
            )
        }
        _ => Task::none(),
    }
}
