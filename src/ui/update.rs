use crate::ui::state::MainWindow;
use crate::ui::messages::{Message, StatusSeverity};
use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::{WorkerStatus};
use crate::models::{ConnectionResult, Device, Filter, OperationResult, PEQData, snap_freq_to_iso, snap_q_to_iso, MAX_BAND_GAIN, MIN_BAND_GAIN, MAX_FREQ, MIN_FREQ, MAX_Q, MIN_Q};
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::ui::state::{ConfirmAction, ConnectionStatus, DisconnectReason, InputBuffer};
use crate::autoeq;
use iced::{clipboard, Task};
use std::sync::Arc;
use crate::ui::main_window::{parse_freq_string, APP_VERSION};

pub fn update(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        // handle_connection
        Message::ClearStatusMessage |
        Message::DismissConfirmDialog |
        Message::DeviceSelected(_) |
        Message::ConnectPressed(_) |
        Message::ConfirmElevatedConnect(_) |
        Message::DisconnectPressed |
        Message::WorkerConnected(..) |
        Message::WorkerDisconnected(..) |
        Message::WorkerStatus(..) |
        Message::Tick(..) => handle_connection(window, message),

        // handle_hardware
        Message::PullPressed |
        Message::PushPressed |
        Message::WorkerPulled(..) |
        Message::WorkerPushed(..) => handle_hardware(window, message),

        // handle_editor
        Message::BandGainChanged(..) |
        Message::BandFreqChanged(..) |
        Message::BandQChanged(..) |
        Message::BandTypeChanged(..) |
        Message::BandGainInput(..) |
        Message::BandFreqInput(..) |
        Message::BandQInput(..) |
        Message::BandFreqSliderChanged(..) |
        Message::BandFreqInputCommit(..) |
        Message::BandGainInputCommit(..) |
        Message::BandQInputCommit(..) |
        Message::BandFreqInputCancel(..) |
        Message::BandGainInputCancel(..) |
        Message::BandQInputCancel(..) |
        Message::GlobalGainChanged(..) |
        Message::ResetFiltersPressed |
        Message::ConfirmResetFilters => handle_editor(window, message),

        // handle_autoeq
        Message::ImportFromClipboard |
        Message::ImportClipboardReceived(..) |
        Message::ImportClipboardFailed(..) |
        Message::ExportAutoEQPressed |
        Message::ExportComplete |
        Message::ToggleDiagnosticsErrorsOnly(..) |
        Message::CopyDiagnostics |
        Message::ClearDiagnostics |
        Message::ExportDiagnosticsToFile |
        Message::DiagnosticsExported(..) => handle_autoeq(window, message),

        // handle_profiles
        Message::ProfilesLoaded(..) |
        Message::ProfileSelected(..) |
        Message::ProfileNameInput(..) |
        Message::SaveProfilePressed |
        Message::ConfirmDeleteProfile |
        Message::DeleteProfilePressed |
        Message::ImportFromFilePressed |
        Message::ExportToFilePressed |
        Message::FileImported(..) |
        Message::FileExported(..) => handle_profiles(window, message),
    }
}

fn handle_connection(window: &mut MainWindow, message: Message) -> Task<Message> {
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
                let connect_task = Task::perform(
                    async move {
                        let rx = worker.connect(Some(device), Some(crate::hardware::worker::BackendKind::Local));
                        rx.recv().unwrap_or_else(|_| ConnectionResult {
                            success: false,
                            device: None,
                            error: Some(AppError::new(ErrorKind::NotConnected, "Channel closed")),
                        })
                    },
                    Message::WorkerConnected,
                );
                connect_task
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
                        let backend = Some(crate::hardware::worker::BackendKind::Elevated);
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

fn handle_hardware(window: &mut MainWindow, message: Message) -> Task<Message> {
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
                let worker = Arc::clone(window.worker.as_ref().unwrap());
                let pull_task = Task::perform(
                    async move {
                        let rx = worker.pull_peq();
                        rx.recv_timeout(std::time::Duration::from_secs(30)).unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Worker closed or timed out")),
                        })
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
                let worker = Arc::clone(window.worker.as_ref().unwrap());
                let filters = window.editor_state.filters.clone();
                let global_gain = window.editor_state.global_gain;
                let push_task = Task::perform(
                    async move {
                        use crate::models::PushPayload;
                        let payload = PushPayload {
                            filters,
                            global_gain: Some(global_gain),
                        };
                        let rx = worker.push_peq(payload);
                        rx.recv_timeout(std::time::Duration::from_secs(30)).unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Worker closed or timed out")),
                        })
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

fn handle_editor(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
                    Message::BandFreqChanged(index, freq) => {
                if let Some(band) = window.editor_state.filters.get_mut(index) {
                    band.freq = freq;
                    band.enabled = true;
                    band.clamp();
                }
                Task::none()
            }
                    Message::BandTypeChanged(index, t) => {
                if let Some(band) = window.editor_state.filters.get_mut(index) {
                    band.filter_type = t;
                }
                Task::none()
            }
                    Message::BandFreqInput(index, s) => {
                window.editor_state.input_buffer.editing_freq = Some((index, s));
                Task::none()
            }
                    Message::BandGainInput(index, s) => {
                window.editor_state.input_buffer.editing_gain = Some((index, s));
                Task::none()
            }
                    Message::BandQInput(index, s) => {
                window.editor_state.input_buffer.editing_q = Some((index, s));
                Task::none()
            }
                    Message::BandFreqInputCommit(index) => {
                if let Some((i, s)) = window.editor_state.input_buffer.editing_freq.take() {
                    if i == index {
                        if let Some(band) = window.editor_state.filters.get_mut(index) {
                            if let Some(v) = parse_freq_string(&s) {
                                band.freq = v.clamp(MIN_FREQ, MAX_FREQ);
                                band.enabled = true;
                                window.editor_state.input_buffer.freq_error = None;
                            } else {
                                window.editor_state.input_buffer.freq_error =
                                    Some((index, "Freq: 20-20000 Hz".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
                    Message::BandGainInputCommit(index) => {
                if let Some((i, s)) = window.editor_state.input_buffer.editing_gain.take() {
                    if i == index {
                        if let Some(band) = window.editor_state.filters.get_mut(index) {
                            if let Ok(v) = s.trim().parse::<f64>() {
                                if v >= MIN_BAND_GAIN && v <= MAX_BAND_GAIN {
                                    band.gain = v;
                                    band.enabled = true;
                                    window.editor_state.input_buffer.gain_error = None;
                                } else {
                                    window.editor_state.input_buffer.gain_error = Some((
                                        index,
                                        format!(
                                            "Gain: {:.0} to {:.0}",
                                            MIN_BAND_GAIN, MAX_BAND_GAIN
                                        ),
                                    ));
                                }
                            } else {
                                window.editor_state.input_buffer.gain_error =
                                    Some((index, "Gain: enter number".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
                    Message::BandQInputCommit(index) => {
                if let Some((i, s)) = window.editor_state.input_buffer.editing_q.take() {
                    if i == index {
                        if let Some(band) = window.editor_state.filters.get_mut(index) {
                            if let Ok(v) = s.trim().parse::<f64>() {
                                if v >= MIN_Q && v <= MAX_Q {
                                    band.q = v;
                                    band.enabled = true;
                                    window.editor_state.input_buffer.q_error = None;
                                } else {
                                    window.editor_state.input_buffer.q_error =
                                        Some((index, format!("Q: {:.1} to {:.1}", MIN_Q, MAX_Q)));
                                }
                            } else {
                                window.editor_state.input_buffer.q_error =
                                    Some((index, "Q: enter number".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
                    Message::BandFreqInputCancel(index) => {
                if let Some((i, _)) = window.editor_state.input_buffer.editing_freq.take() {
                    if i == index {}
                }
                Task::none()
            }
                    Message::BandGainInputCancel(index) => {
                if let Some((i, _)) = window.editor_state.input_buffer.editing_gain.take() {
                    if i == index {}
                }
                Task::none()
            }
                    Message::BandQInputCancel(index) => {
                if let Some((i, _)) = window.editor_state.input_buffer.editing_q.take() {
                    if i == index {}
                }
                Task::none()
            }
                    Message::BandFreqSliderChanged(index, v) => {
                // v is log10(freq) - convert back and snap to ISO
                if let Some(band) = window.editor_state.filters.get_mut(index) {
                    let hz = 10f64.powf(v).round() as u16;
                    band.freq = snap_freq_to_iso(hz);
                }
                Task::none()
            }
                    Message::BandGainChanged(index, v) => {
                if let Some(band) = window.editor_state.filters.get_mut(index) {
                    band.gain = v.clamp(MIN_BAND_GAIN, MAX_BAND_GAIN);
                    band.enabled = true;
                }
                Task::none()
            }
                    Message::BandQChanged(index, v) => {
                if let Some(band) = window.editor_state.filters.get_mut(index) {
                    // v is log scale Q - convert and snap to ISO
                    let q_val = 10f64.powf(v);
                    band.q = snap_q_to_iso(q_val);
                }
                Task::none()
            }
                    Message::GlobalGainChanged(gain) => {
                window.editor_state.global_gain = gain;
                Task::none()
            }
                    Message::ResetFiltersPressed => {
                window.editor_state.pending_confirm = ConfirmAction::ResetFilters;
                Task::none()
            }
                    Message::ConfirmResetFilters => {
                if matches!(
                    window.editor_state.pending_confirm,
                    ConfirmAction::ResetFilters
                ) {
                    let default_filters: Vec<Filter> =
                        (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
                    window.editor_state.filters = default_filters;
                    window.editor_state.global_gain = 0;
                    window.editor_state.input_buffer = InputBuffer::default();
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        "Reset filters to default",
                    ));
                    window.editor_state.pending_confirm = ConfirmAction::None;
                    window.set_status("Filters reset to default", StatusSeverity::Info)
                } else {
                    Task::none()
                }
            }
        _ => Task::none(),
    }
}

fn handle_autoeq(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
                    Message::ExportAutoEQPressed => {
                let peq = PEQData {
                    filters: window.editor_state.filters.clone(),
                    global_gain: window.editor_state.global_gain,
                };
                let output = autoeq::peq_to_autoeq(&peq);
                let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
                let status_task = window.set_status("Exported to clipboard", StatusSeverity::Success);
                Task::batch(vec![write_task, status_task])
            }
                    Message::ExportComplete => Task::none(),
                    Message::ImportFromClipboard => {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::AutoEQ,
                    "Import from clipboard started",
                ));
                clipboard::read().map(|result| match result {
                    Some(text) => Message::ImportClipboardReceived(text),
                    None => Message::ImportClipboardFailed("Clipboard empty or not text".into()),
                })
            }
                    Message::ImportClipboardReceived(text) => match autoeq::parse_autoeq_text(&text) {
                Ok(peq) => {
                    let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
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
                        Source::AutoEQ,
                        format!("Import successful: {} filters", enabled_count),
                    ));
                    window.set_status(
                        format!("Imported {} filters", enabled_count),
                        StatusSeverity::Success,
                    )
                }
                Err(e) => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::AutoEQ,
                        format!("Import failed: {}", e),
                    ));
                    window.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
                }
            },
                    Message::ImportClipboardFailed(msg) => {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::AutoEQ,
                    msg.clone(),
                ));
                window.set_status(msg, StatusSeverity::Error)
            }
                    Message::CopyDiagnostics => {
                let conn_str = format!("{:?}", window.connection_status);
                let output = crate::diagnostics::format_diagnostics(
                    &window.diagnostics,
                    APP_VERSION,
                    &conn_str,
                );
                let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
                let status_task = window.set_status("Diagnostics copied", StatusSeverity::Info);
                Task::batch(vec![write_task, status_task])
            }
                    Message::ClearDiagnostics => {
                window.diagnostics.clear();
                window.set_status("Diagnostics cleared", StatusSeverity::Info)
            }
                    Message::ToggleDiagnosticsErrorsOnly(v) => {
                window.editor_state.diagnostics_errors_only = v;
                Task::none()
            }
                    Message::ExportDiagnosticsToFile => {
                let conn_str = format!("{:?}", window.connection_status);
                let output = crate::diagnostics::format_diagnostics(
                    &window.diagnostics,
                    APP_VERSION,
                    &conn_str,
                );
                let now = chrono::Local::now();
                let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
                let path = dirs::document_dir()
                    .or_else(dirs::data_dir)
                    .unwrap_or_else(std::env::temp_dir)
                    .join(&filename);
                match std::fs::write(&path, output) {
                    Ok(_) => Task::done(Message::DiagnosticsExported(path.display().to_string())),
                    Err(e) => {
                        window.set_status(format!("Export failed: {}", e), StatusSeverity::Error)
                    }
                }
            }
                    Message::DiagnosticsExported(name) => {
                window.set_status(format!("Saved to {}", name), StatusSeverity::Success)
            }
        _ => Task::none(),
    }
}

fn handle_profiles(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
                    Message::ProfilesLoaded(result) => {
                match result {
                    Ok(profiles) => {
                        window.editor_state.profiles = profiles;
                        Task::none()
                    }
                    Err(e) => {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Failed to load profiles: {}", e),
                        ));
                        window.set_status(format!("Failed to load profiles: {}", e), StatusSeverity::Error)
                    }
                }
            }

                    Message::ProfileSelected(name) => {
                if let Some(profile) = window.editor_state.profiles.iter().find(|p| p.name == name) {
                    window.editor_state.filters = profile
                        .data
                        .filters
                        .clone()
                        .into_iter()
                        .map(|mut f| {
                            f.enabled = true;
                            f
                        })
                        .collect();
                    window.editor_state.global_gain = profile.data.global_gain;
                    window.editor_state.selected_profile_name = Some(name);
                    window.editor_state.new_profile_name = profile.name.clone();
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Loaded profile: {}", profile.name),
                    ));
                    window.set_status(
                        format!("Loaded profile: {}", profile.name),
                        StatusSeverity::Info,
                    )
                } else {
                    Task::none()
                }
            }
                    Message::ProfileNameInput(name) => {
                window.editor_state.new_profile_name = name;
                Task::none()
            }
                    Message::SaveProfilePressed => {
                let name = window.editor_state.new_profile_name.trim().to_string();
                if name.is_empty() {
                    return window.set_status("Invalid profile name", StatusSeverity::Warning);
                }
                let data = PEQData {
                    filters: window.editor_state.filters.clone(),
                    global_gain: window.editor_state.global_gain,
                };
                match crate::storage::save_profile(&name, &data) {
                    Ok(_) => {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Saved profile: {}", name),
                        ));
                        let reload_task = Task::perform(
                            async move { crate::storage::load_all_profiles() },
                            Message::ProfilesLoaded,
                        );
                        let status_task = window.set_status(
                            format!("Saved profile: {}", name),
                            StatusSeverity::Success,
                        );
                        Task::batch(vec![reload_task, status_task])
                    }
                    Err(e) => {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Save failed: {}", e),
                        ));
                        window.set_status(format!("Failed to save: {}", e), StatusSeverity::Error)
                    }
                }
            }
                    Message::DeleteProfilePressed => {
                window.editor_state.pending_confirm = ConfirmAction::DeleteProfile;
                Task::none()
            }
                    Message::ConfirmDeleteProfile => {
                if matches!(
                    window.editor_state.pending_confirm,
                    ConfirmAction::DeleteProfile
                ) {
                    let name = match &window.editor_state.selected_profile_name {
                        Some(n) => n.clone(),
                        None => return Task::none(),
                    };
                    match crate::storage::delete_profile(&name) {
                        Ok(_) => {
                            window.editor_state.selected_profile_name = None;
                            window.editor_state.new_profile_name = String::new();
                            window.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::UI,
                                format!("Deleted profile: {}", name),
                            ));
                            let reload_task = Task::perform(
                                async move { crate::storage::load_all_profiles() },
                                Message::ProfilesLoaded,
                            );
                            let status_task = window.set_status(
                                format!("Deleted profile: {}", name),
                                StatusSeverity::Success,
                            );
                            window.editor_state.pending_confirm = ConfirmAction::None;
                            Task::batch(vec![reload_task, status_task])
                        }
                        Err(e) => {
                            window.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Error,
                                Source::UI,
                                format!("Delete failed: {}", e),
                            ));
                            window.set_status(format!("Failed to delete: {}", e), StatusSeverity::Error)
                        }
                    }
                } else {
                    Task::none()
                }
            }
                    Message::ImportFromFilePressed => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Frost-Tune Profile", &["json", "txt"])
                            .pick_file()
                            .await
                    },
                    |handle| Message::FileImported(handle.map(|h| h.path().to_path_buf())),
                )
            }
                    Message::FileImported(path_opt) => {
                if let Some(path) = path_opt {
                    match crate::storage::import_profile(&path) {
                        Ok(profile) => {
                            window.editor_state.profiles.push(profile.clone());
                            window.editor_state.selected_profile_name = Some(profile.name.clone());
                            window.editor_state.new_profile_name = profile.name.clone();
                            window.editor_state.filters = profile.data.filters.clone();
                            window.editor_state.global_gain = profile.data.global_gain;
                            window.set_status(
                                format!("Imported profile: {}", profile.name),
                                StatusSeverity::Success,
                            )
                        }
                        Err(e) => window.set_status(format!("Import failed: {}", e), StatusSeverity::Error),
                    }
                } else {
                    Task::none()
                }
            }
                    Message::ExportToFilePressed => {
                let peq = PEQData {
                    filters: window.editor_state.filters.clone(),
                    global_gain: window.editor_state.global_gain,
                };
                let name = if window.editor_state.new_profile_name.is_empty() {
                    "profile".to_string()
                } else {
                    window.editor_state.new_profile_name.clone()
                };

                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Frost-Tune Profile", &["json", "txt"])
                            .set_file_name(&format!("{}.txt", name))
                            .save_file()
                            .await
                    },
                    move |handle| Message::FileExported(handle.map(|h| h.path().to_path_buf()), peq),
                )
            }
                    Message::FileExported(path_opt, peq) => {
                if let Some(path) = path_opt {
                    match crate::storage::export_profile(&path, &peq) {
                        Ok(_) => window.set_status("Profile exported", StatusSeverity::Success),
                        Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
                    }
                } else {
                    Task::none()
                }
            }
        _ => Task::none(),
    }
}
