use crate::autoeq;
use crate::diagnostics::{DiagnosticEvent, DiagnosticsStore, LogLevel, Source};
use crate::error::{AppError, ErrorKind};
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use crate::models::{
    snap_freq_to_iso, snap_q_to_iso, ConnectionResult, Device, Filter, OperationResult, PEQData,
    MAX_BAND_GAIN, MAX_FREQ, MAX_Q, MIN_BAND_GAIN, MIN_FREQ, MIN_Q,
};
use crate::ui::messages::{Message, StatusMessage, StatusSeverity};
use crate::ui::state::{
    ConfirmAction, ConnectionStatus, DisconnectReason, EditorState, InputBuffer, MainWindow,
    OperationLock,
};
use crate::ui::theme;
use crate::ui::tokens::{SPACE_8, SPACE_16, SPACE_24, WINDOW_MEDIUM_MAX, WINDOW_NARROW_MAX};
use crate::ui::views;
use iced::{
    clipboard,
    widget::{column, container, responsive, row, scrollable},
    Element, Length, Padding, Subscription, Task,
};
use std::sync::Arc;

pub const STATUS_AUTO_CLEAR_SECS: u64 = 5;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutBucket {
    Narrow,
    Medium,
    Wide,
}

pub fn layout_bucket_for_width(width: f32) -> LayoutBucket {
    if width <= WINDOW_NARROW_MAX {
        LayoutBucket::Narrow
    } else if width <= WINDOW_MEDIUM_MAX {
        LayoutBucket::Medium
    } else {
        LayoutBucket::Wide
    }
}

fn parse_freq_string(s: &str) -> Option<u16> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let mut multiplier = 1.0;
    let mut num_str = s.as_str();

    if s.ends_with('k') {
        multiplier = 1000.0;
        num_str = &s[..s.len() - 1].trim();
    } else if s.ends_with("hz") {
        num_str = &s[..s.len() - 2].trim();
    }

    if let Ok(v) = num_str.parse::<f64>() {
        let hz = (v * multiplier).round() as u16;
        if hz >= 20 && hz <= 20000 {
            return Some(hz);
        }
    }
    None
}

impl MainWindow {
    fn new() -> (Self, Task<Message>) {
        let worker = Arc::new(UsbWorker::new());
        let default_filters: Vec<Filter> =
            (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let window = MainWindow {
            connection_status: ConnectionStatus::Disconnected,
            disconnect_reason: DisconnectReason::None,
            editor_state: EditorState {
                filters: default_filters.clone(),
                global_gain: 0,
                status_message: None,
                diagnostics_errors_only: false,
                profiles: Vec::new(),
                selected_profile_name: None,
                new_profile_name: String::new(),
                input_buffer: InputBuffer::default(),
                advanced_filters_expanded: false,
                diagnostics_expanded: false,
                pending_confirm: ConfirmAction::None,
            },
            operation_lock: OperationLock::default(),
            worker: Some(worker),
            connected_device: None,
            diagnostics: DiagnosticsStore::default(),
        };
        let load_profiles_task = Task::perform(
            async move { crate::storage::load_all_profiles().unwrap_or_default() },
            Message::ProfilesLoaded,
        );
        let load_prefs_task = Task::perform(
            async move { crate::storage::load_ui_preferences().unwrap_or_default() },
            Message::UiPreferencesLoaded,
        );
        (
            window,
            Task::batch(vec![load_profiles_task, load_prefs_task]),
        )
    }

    fn title(&self) -> String {
        "Frost-Tune".into()
    }

    fn app_theme(_state: &Self) -> iced::Theme {
        theme::theme()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ClearStatusMessage => {
                self.editor_state.status_message = None;
                Task::none()
            }
            Message::DismissConfirmDialog => {
                self.editor_state.pending_confirm = ConfirmAction::None;
                Task::none()
            }
            Message::ConnectPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                self.connection_status = ConnectionStatus::Connecting;
                self.operation_lock.is_connecting = true;
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Connect pressed",
                ));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                let connect_task = Task::perform(
                    async move {
                        let rx = worker.connect();
                        rx.recv().unwrap_or(ConnectionResult {
                            success: false,
                            device: None,
                            error: Some("Worker closed".into()),
                        })
                    },
                    Message::WorkerConnected,
                );
                let status_task = self.set_status("Connecting to device...", StatusSeverity::Info);
                Task::batch(vec![connect_task, status_task])
            }
            Message::DisconnectPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                self.disconnect_reason = DisconnectReason::Manual;
                self.operation_lock.is_disconnecting = true;
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Disconnect pressed",
                ));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
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
                let status_task = self.set_status("Disconnecting...", StatusSeverity::Info);
                Task::batch(vec![disconnect_task, status_task])
            }
            Message::PullPressed => {
                if self.worker.is_none()
                    || self.operation_lock.is_pulling
                    || self.operation_lock.is_pushing
                    || self.operation_lock.is_connecting
                {
                    return Task::none();
                }
                self.operation_lock.is_pulling = true;
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Pull pressed",
                ));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                let pull_task = Task::perform(
                    async move {
                        let rx = worker.pull_peq();
                        rx.recv().unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Worker closed")),
                        })
                    },
                    Message::WorkerPulled,
                );
                let status_task = self.set_status("Reading from device...", StatusSeverity::Info);
                Task::batch(vec![pull_task, status_task])
            }
            Message::PushPressed => {
                if self.worker.is_none()
                    || self.operation_lock.is_pulling
                    || self.operation_lock.is_pushing
                    || self.operation_lock.is_connecting
                {
                    return Task::none();
                }
                self.operation_lock.is_pushing = true;
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Push pressed",
                ));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                let filters = self.editor_state.filters.clone();
                let global_gain = self.editor_state.global_gain;
                let push_task = Task::perform(
                    async move {
                        use crate::models::PushPayload;
                        let payload = PushPayload {
                            filters,
                            global_gain: Some(global_gain),
                        };
                        let rx = worker.push_peq(payload);
                        rx.recv().unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some(AppError::new(ErrorKind::Unknown, "Worker closed")),
                        })
                    },
                    Message::WorkerPushed,
                );
                let status_task = self.set_status("Writing to device...", StatusSeverity::Info);
                Task::batch(vec![push_task, status_task])
            }
            Message::WorkerConnected(result) => {
                self.operation_lock.is_connecting = false;
                let device_name_owned = if let Some(ref d) = result.device {
                    Device::from_vid_pid(d.vendor_id, d.product_id)
                        .name()
                        .to_string()
                } else {
                    "Unknown Device".to_string()
                };

                if result.success {
                    self.connection_status = ConnectionStatus::Connected;
                    self.connected_device = result.device;
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        format!("Connected to {}", device_name_owned),
                    ));
                    self.set_status(
                        format!("Connected to {}", device_name_owned),
                        StatusSeverity::Success,
                    )
                } else {
                    let err = result.error.unwrap_or_else(|| AppError::new(ErrorKind::Unknown, "Unknown error"));
                    let user_error = match err.kind {
                        ErrorKind::PolkitAuthRequired => "Authentication required to access USB DAC on Linux. Approve the polkit prompt and retry.".to_string(),
                        _ => err.message.clone(),
                    };
                    self.connection_status = ConnectionStatus::Error(user_error.clone());
                    self.connected_device = None;
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Connect failed: {}", err.message),
                    ));
                    self.set_status(
                        format!("Connect failed: {}", user_error),
                        StatusSeverity::Error,
                    )
                }
            }
            Message::WorkerDisconnected(_) => {
                self.operation_lock.is_disconnecting = false;
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Disconnected",
                ));
                self.connection_status = ConnectionStatus::Disconnected;
                self.connected_device = None;
                self.set_status("Disconnected", StatusSeverity::Info)
            }
            Message::WorkerPulled(result) => {
                self.operation_lock.is_pulling = false;
                if result.success {
                    if let Some(peq) = result.data {
                        self.editor_state.filters = peq
                            .filters
                            .into_iter()
                            .map(|mut f| {
                                f.enabled = true;
                                f
                            })
                            .collect();
                        self.editor_state.global_gain = peq.global_gain;
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::Worker,
                            "Pull successful",
                        ));
                        return self
                            .set_status("Data pulled from device", StatusSeverity::Success);
                    }
                    Task::none()
                } else if let Some(err) = result.error {
                    if err.kind == ErrorKind::NotConnected || err.kind == ErrorKind::PolkitAuthRequired {
                        self.connection_status = ConnectionStatus::Disconnected;
                    } else {
                        self.connection_status = ConnectionStatus::Error(err.message.clone());
                    }
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Connection failed: {}", err.message),
                    ));
                    self.set_status(format!("Connection failed: {}", err.message), StatusSeverity::Error)
                } else {
                    Task::none()
                }
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if result.success {
                    if let Some(peq) = result.data {
                        self.editor_state.filters = peq
                            .filters
                            .into_iter()
                            .map(|mut f| {
                                f.enabled = true;
                                f
                            })
                            .collect();
                        self.editor_state.global_gain = peq.global_gain;
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::Worker,
                            "Push successful",
                        ));
                        return self.set_status(
                            "Settings applied and verified",
                            StatusSeverity::Success,
                        );
                    }
                    Task::none()
                } else if let Some(err) = result.error {
                    if err.kind == ErrorKind::NotConnected {
                        self.connection_status = ConnectionStatus::Disconnected;
                    } else {
                        self.connection_status = ConnectionStatus::Error(err.message.clone());
                    }
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Push failed: {}", err.message),
                    ));
                    self.set_status(format!("Push failed: {}", err.message), StatusSeverity::Error)
                } else {
                    Task::none()
                }
            }
            Message::WorkerStatus(status) => {
                self.connected_device = if status.connected {
                    status.device.clone()
                } else {
                    None
                };

                if status.connected && self.connection_status != ConnectionStatus::Connected {
                    self.connection_status = ConnectionStatus::Connected;
                    self.disconnect_reason = DisconnectReason::None;
                    log::info!("Device connected");
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Device connected (poll)",
                    ));
                } else if !status.connected && self.connection_status == ConnectionStatus::Connected
                {
                    self.connection_status = ConnectionStatus::Disconnected;
                    self.disconnect_reason = DisconnectReason::DeviceLost;
                    log::info!("Device disconnected");
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Device disconnected (poll)",
                    ));
                }
                Task::none()
            }
            Message::Tick(_) => {
                let worker = match &self.worker {
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
                        })
                    },
                    Message::WorkerStatus,
                )
            }
            Message::BandFreqChanged(index, freq) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.freq = freq;
                    band.enabled = true;
                    band.clamp();
                }
                Task::none()
            }
            Message::BandTypeChanged(index, t) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.filter_type = t;
                }
                Task::none()
            }
            Message::BandFreqInput(index, s) => {
                self.editor_state.input_buffer.editing_freq = Some((index, s));
                Task::none()
            }
            Message::BandGainInput(index, s) => {
                self.editor_state.input_buffer.editing_gain = Some((index, s));
                Task::none()
            }
            Message::BandQInput(index, s) => {
                self.editor_state.input_buffer.editing_q = Some((index, s));
                Task::none()
            }
            Message::BandFreqInputCommit(index) => {
                if let Some((i, s)) = self.editor_state.input_buffer.editing_freq.take() {
                    if i == index {
                        if let Some(band) = self.editor_state.filters.get_mut(index) {
                            if let Some(v) = parse_freq_string(&s) {
                                band.freq = v.clamp(MIN_FREQ, MAX_FREQ);
                                band.enabled = true;
                                self.editor_state.input_buffer.freq_error = None;
                            } else {
                                self.editor_state.input_buffer.freq_error =
                                    Some((index, "Freq: 20-20000 Hz".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::BandGainInputCommit(index) => {
                if let Some((i, s)) = self.editor_state.input_buffer.editing_gain.take() {
                    if i == index {
                        if let Some(band) = self.editor_state.filters.get_mut(index) {
                            if let Ok(v) = s.trim().parse::<f64>() {
                                if v >= MIN_BAND_GAIN && v <= MAX_BAND_GAIN {
                                    band.gain = v;
                                    band.enabled = true;
                                    self.editor_state.input_buffer.gain_error = None;
                                } else {
                                    self.editor_state.input_buffer.gain_error = Some((
                                        index,
                                        format!(
                                            "Gain: {:.0} to {:.0}",
                                            MIN_BAND_GAIN, MAX_BAND_GAIN
                                        ),
                                    ));
                                }
                            } else {
                                self.editor_state.input_buffer.gain_error =
                                    Some((index, "Gain: enter number".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::BandQInputCommit(index) => {
                if let Some((i, s)) = self.editor_state.input_buffer.editing_q.take() {
                    if i == index {
                        if let Some(band) = self.editor_state.filters.get_mut(index) {
                            if let Ok(v) = s.trim().parse::<f64>() {
                                if v >= MIN_Q && v <= MAX_Q {
                                    band.q = v;
                                    band.enabled = true;
                                    self.editor_state.input_buffer.q_error = None;
                                } else {
                                    self.editor_state.input_buffer.q_error =
                                        Some((index, format!("Q: {:.1} to {:.1}", MIN_Q, MAX_Q)));
                                }
                            } else {
                                self.editor_state.input_buffer.q_error =
                                    Some((index, "Q: enter number".to_string()));
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::BandFreqInputCancel(index) => {
                if let Some((i, _)) = self.editor_state.input_buffer.editing_freq.take() {
                    if i == index {}
                }
                Task::none()
            }
            Message::BandGainInputCancel(index) => {
                if let Some((i, _)) = self.editor_state.input_buffer.editing_gain.take() {
                    if i == index {}
                }
                Task::none()
            }
            Message::BandQInputCancel(index) => {
                if let Some((i, _)) = self.editor_state.input_buffer.editing_q.take() {
                    if i == index {}
                }
                Task::none()
            }
            Message::BandFreqSliderChanged(index, v) => {
                // v is log10(freq) - convert back and snap to ISO
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    let hz = 10f64.powf(v).round() as u16;
                    band.freq = snap_freq_to_iso(hz);
                }
                Task::none()
            }
            Message::BandGainChanged(index, v) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.gain = v.clamp(MIN_BAND_GAIN, MAX_BAND_GAIN);
                    band.enabled = true;
                }
                Task::none()
            }
            Message::BandQChanged(index, v) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    // v is log scale Q - convert and snap to ISO
                    let q_val = 10f64.powf(v);
                    band.q = snap_q_to_iso(q_val);
                }
                Task::none()
            }
            Message::GlobalGainChanged(gain) => {
                self.editor_state.global_gain = gain;
                Task::none()
            }
            Message::ResetFiltersPressed => {
                self.editor_state.pending_confirm = ConfirmAction::ResetFilters;
                Task::none()
            }
            Message::ConfirmResetFilters => {
                if matches!(
                    self.editor_state.pending_confirm,
                    ConfirmAction::ResetFilters
                ) {
                    let default_filters: Vec<Filter> =
                        (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
                    self.editor_state.filters = default_filters;
                    self.editor_state.global_gain = 0;
                    self.editor_state.input_buffer = InputBuffer::default();
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        "Reset filters to default",
                    ));
                    self.editor_state.pending_confirm = ConfirmAction::None;
                    self.set_status("Filters reset to default", StatusSeverity::Info)
                } else {
                    Task::none()
                }
            }
            Message::ExportAutoEQPressed => {
                let peq = PEQData {
                    filters: self.editor_state.filters.clone(),
                    global_gain: self.editor_state.global_gain,
                };
                let output = autoeq::peq_to_autoeq(&peq);
                let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
                let status_task = self.set_status("Exported to clipboard", StatusSeverity::Success);
                Task::batch(vec![write_task, status_task])
            }
            Message::ExportComplete => Task::none(),
            Message::ImportFromClipboard => {
                self.diagnostics.push(DiagnosticEvent::new(
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
                    self.editor_state.filters = peq
                        .filters
                        .into_iter()
                        .map(|mut f| {
                            f.enabled = true;
                            f
                        })
                        .collect();
                    self.editor_state.global_gain = peq.global_gain;
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::AutoEQ,
                        format!("Import successful: {} filters", enabled_count),
                    ));
                    self.set_status(
                        format!("Imported {} filters", enabled_count),
                        StatusSeverity::Success,
                    )
                }
                Err(e) => {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::AutoEQ,
                        format!("Import failed: {}", e),
                    ));
                    self.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
                }
            },
            Message::ImportClipboardFailed(msg) => {
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::AutoEQ,
                    msg.clone(),
                ));
                self.set_status(msg, StatusSeverity::Error)
            }
            Message::CopyDiagnostics => {
                let conn_str = format!("{:?}", self.connection_status);
                let output = crate::diagnostics::format_diagnostics(
                    &self.diagnostics,
                    APP_VERSION,
                    &conn_str,
                );
                let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
                let status_task = self.set_status("Diagnostics copied", StatusSeverity::Info);
                Task::batch(vec![write_task, status_task])
            }
            Message::ClearDiagnostics => {
                self.diagnostics.clear();
                self.set_status("Diagnostics cleared", StatusSeverity::Info)
            }
            Message::ToggleDiagnosticsErrorsOnly(v) => {
                self.editor_state.diagnostics_errors_only = v;
                Task::none()
            }
            Message::ExportDiagnosticsToFile => {
                let conn_str = format!("{:?}", self.connection_status);
                let output = crate::diagnostics::format_diagnostics(
                    &self.diagnostics,
                    APP_VERSION,
                    &conn_str,
                );
                let now = chrono::Local::now();
                let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
                let path = std::path::PathBuf::from(&filename);
                match std::fs::write(&path, output) {
                    Ok(_) => Task::done(Message::DiagnosticsExported(filename)),
                    Err(e) => {
                        self.set_status(format!("Export failed: {}", e), StatusSeverity::Error)
                    }
                }
            }
            Message::DiagnosticsExported(name) => {
                self.set_status(format!("Saved to {}", name), StatusSeverity::Success)
            }
            Message::ProfilesLoaded(profiles) => {
                self.editor_state.profiles = profiles;
                Task::none()
            }
            Message::UiPreferencesLoaded(prefs) => {
                self.editor_state.advanced_filters_expanded = prefs.advanced_filters_expanded;
                self.editor_state.diagnostics_expanded = prefs.diagnostics_expanded;
                Task::none()
            }
            Message::ProfileSelected(name) => {
                if let Some(profile) = self.editor_state.profiles.iter().find(|p| p.name == name) {
                    self.editor_state.filters = profile
                        .data
                        .filters
                        .clone()
                        .into_iter()
                        .map(|mut f| {
                            f.enabled = true;
                            f
                        })
                        .collect();
                    self.editor_state.global_gain = profile.data.global_gain;
                    self.editor_state.selected_profile_name = Some(name);
                    self.editor_state.new_profile_name = profile.name.clone();
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Loaded profile: {}", profile.name),
                    ));
                    self.set_status(
                        format!("Loaded profile: {}", profile.name),
                        StatusSeverity::Info,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ProfileNameInput(name) => {
                self.editor_state.new_profile_name = name;
                Task::none()
            }
            Message::SaveProfilePressed => {
                let name = self.editor_state.new_profile_name.trim().to_string();
                if name.is_empty() {
                    return self.set_status("Invalid profile name", StatusSeverity::Warning);
                }
                let data = PEQData {
                    filters: self.editor_state.filters.clone(),
                    global_gain: self.editor_state.global_gain,
                };
                match crate::storage::save_profile(&name, &data) {
                    Ok(_) => {
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Saved profile: {}", name),
                        ));
                        let reload_task = Task::perform(
                            async move { crate::storage::load_all_profiles().unwrap_or_default() },
                            Message::ProfilesLoaded,
                        );
                        let status_task = self.set_status(
                            format!("Saved profile: {}", name),
                            StatusSeverity::Success,
                        );
                        Task::batch(vec![reload_task, status_task])
                    }
                    Err(e) => {
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Save failed: {}", e),
                        ));
                        self.set_status(format!("Failed to save: {}", e), StatusSeverity::Error)
                    }
                }
            }
            Message::DeleteProfilePressed => {
                self.editor_state.pending_confirm = ConfirmAction::DeleteProfile;
                Task::none()
            }
            Message::ConfirmDeleteProfile => {
                if matches!(
                    self.editor_state.pending_confirm,
                    ConfirmAction::DeleteProfile
                ) {
                    let name = match &self.editor_state.selected_profile_name {
                        Some(n) => n.clone(),
                        None => return Task::none(),
                    };
                    match crate::storage::delete_profile(&name) {
                        Ok(_) => {
                            self.editor_state.selected_profile_name = None;
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::UI,
                                format!("Deleted profile: {}", name),
                            ));
                            let reload_task = Task::perform(
                                async move { crate::storage::load_all_profiles().unwrap_or_default() },
                                Message::ProfilesLoaded,
                            );
                            let status_task = self.set_status(
                                format!("Deleted profile: {}", name),
                                StatusSeverity::Info,
                            );
                            self.editor_state.pending_confirm = ConfirmAction::None;
                            Task::batch(vec![reload_task, status_task])
                        }
                        Err(e) => {
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Error,
                                Source::UI,
                                format!("Delete failed: {}", e),
                            ));
                            self.editor_state.pending_confirm = ConfirmAction::None;
                            self.set_status(
                                format!("Failed to delete: {}", e),
                                StatusSeverity::Error,
                            )
                        }
                    }
                } else {
                    Task::none()
                }
            }
            Message::ImportFromFilePressed => {
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Import from file started",
                ));
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Text", &["txt"][..])
                            .pick_file()
                            .await;

                        if let Some(file) = file {
                            let bytes = file.read().await;
                            String::from_utf8(bytes)
                                .map_err(|e| format!("Failed to read file as UTF-8: {}", e))
                        } else {
                            Err("Cancelled".to_string())
                        }
                    },
                    Message::FileImported,
                )
            }
            Message::ExportToFilePressed => {
                let peq = PEQData {
                    filters: self.editor_state.filters.clone(),
                    global_gain: self.editor_state.global_gain,
                };
                let output = autoeq::peq_to_autoeq(&peq);
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Text", &["txt"][..])
                            .set_file_name("profile.txt")
                            .save_file()
                            .await;

                        if let Some(file) = file {
                            file.write(output.as_bytes())
                                .await
                                .map(|_| file.file_name())
                                .map_err(|e| format!("Failed to write file: {}", e))
                        } else {
                            Err("Cancelled".to_string())
                        }
                    },
                    Message::FileExported,
                )
            }
            Message::FileImported(result) => match result {
                Ok(text) => match autoeq::parse_autoeq_text(&text) {
                    Ok(peq) => {
                        let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
                        self.editor_state.filters = peq
                            .filters
                            .into_iter()
                            .map(|mut f| {
                                f.enabled = true;
                                f
                            })
                            .collect();
                        self.editor_state.global_gain = peq.global_gain;
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::AutoEQ,
                            format!("Import successful from file: {} filters", enabled_count),
                        ));
                        self.set_status(
                            format!("Imported {} filters from file", enabled_count),
                            StatusSeverity::Success,
                        )
                    }
                    Err(e) => {
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::AutoEQ,
                            format!("Import failed: {}", e),
                        ));
                        self.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
                    }
                },
                Err(e) if e != "Cancelled" => {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::UI,
                        format!("File error: {}", e),
                    ));
                    self.set_status(format!("File error: {}", e), StatusSeverity::Error)
                }
                _ => Task::none(),
            },
            Message::FileExported(result) => match result {
                Ok(name) => {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Exported to {}", name),
                    ));
                    self.set_status(format!("Exported to {}", name), StatusSeverity::Success)
                }
                Err(e) if e != "Cancelled" => {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::UI,
                        format!("Export error: {}", e),
                    ));
                    self.set_status(format!("Export error: {}", e), StatusSeverity::Error)
                }
                _ => Task::none(),
            },
            Message::ToggleAdvancedFilters(expanded) => {
                self.editor_state.advanced_filters_expanded = expanded;
                let prefs = crate::storage::UiPreferences {
                    advanced_filters_expanded: self.editor_state.advanced_filters_expanded,
                    diagnostics_expanded: self.editor_state.diagnostics_expanded,
                };
                if let Err(e) = crate::storage::save_ui_preferences(&prefs) {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::UI,
                        format!("Failed to save UI preferences: {}", e),
                    ));
                }
                Task::none()
            }
            Message::ToggleDiagnosticsExpanded(expanded) => {
                self.editor_state.diagnostics_expanded = expanded;
                let prefs = crate::storage::UiPreferences {
                    advanced_filters_expanded: self.editor_state.advanced_filters_expanded,
                    diagnostics_expanded: self.editor_state.diagnostics_expanded,
                };
                if let Err(e) = crate::storage::save_ui_preferences(&prefs) {
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::UI,
                        format!("Failed to save UI preferences: {}", e),
                    ));
                }
                Task::none()
            }
        }
    }

    pub fn set_status(
        &mut self,
        content: impl Into<String>,
        severity: StatusSeverity,
    ) -> Task<Message> {
        let content = content.into();
        let skip_diag_echo = content.starts_with("Loaded profile:")
            || content.starts_with("Saved profile:")
            || content.starts_with("Deleted profile:")
            || content.starts_with("Imported ")
            || content.starts_with("Exported ");

        if !skip_diag_echo {
            self.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Status set: {}", content),
            ));
        }
        let should_auto_clear = self.status_should_auto_clear(severity);
        self.editor_state.status_message = Some(StatusMessage {
            content,
            severity,
            created_at: chrono::Local::now().to_rfc3339(),
        });
        if should_auto_clear {
            Task::perform(
                async { tokio::time::sleep(Self::status_auto_clear_duration()).await },
                |_| Message::ClearStatusMessage,
            )
        } else {
            Task::none()
        }
    }

    pub fn status_auto_clear_duration() -> std::time::Duration {
        std::time::Duration::from_secs(STATUS_AUTO_CLEAR_SECS)
    }

    pub fn status_should_auto_clear(&self, severity: StatusSeverity) -> bool {
        if self.operation_lock.is_connecting
            || self.operation_lock.is_disconnecting
            || self.operation_lock.is_pulling
            || self.operation_lock.is_pushing
        {
            return false;
        }
        matches!(severity, StatusSeverity::Info | StatusSeverity::Success)
    }

    pub fn header_status_message(&self) -> String {
        match &self.connection_status {
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
            ConnectionStatus::Connecting => "Connecting...".to_string(),
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Error(e) => format!("Error: {}", e),
        }
    }

    pub fn status_banner_message(&self) -> Option<String> {
        self.editor_state
            .status_message
            .as_ref()
            .map(|m| m.content.clone())
    }

    pub fn disabled_reason_for_action(&self, action: &str) -> Option<String> {
        if let ConnectionStatus::Error(e) = &self.connection_status {
            return Some(format!("Error: {}", e));
        }

        match action {
            "connect" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    None
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else {
                    Some("Device already connected or in error".to_string())
                }
            }
            "disconnect" => {
                if self.operation_lock.is_disconnecting {
                    Some("Disconnecting...".to_string())
                } else if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else {
                    None
                }
            }
            "read" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else if self.operation_lock.is_pulling {
                    Some("Operation in progress: Reading".to_string())
                } else if self.operation_lock.is_pushing {
                    Some("Operation in progress: Writing or Connecting".to_string())
                } else {
                    None
                }
            }
            "write" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else if self.operation_lock.is_pushing {
                    Some("Operation in progress: Writing".to_string())
                } else if self.operation_lock.is_pulling {
                    Some("Operation in progress: Reading or Connecting".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn header_disabled_reason_message(&self) -> Option<String> {
        if self.operation_lock.is_disconnecting {
            return Some("Disconnecting...".to_string());
        }
        if self.operation_lock.is_connecting {
            return Some("Connecting to device...".to_string());
        }
        if self.operation_lock.is_pulling {
            return Some("Operation in progress: Reading".to_string());
        }
        if self.operation_lock.is_pushing {
            return Some("Operation in progress: Writing or Connecting".to_string());
        }
        if let ConnectionStatus::Error(e) = &self.connection_status {
            return Some(format!("Error: {}", e));
        }
        if self.connection_status == ConnectionStatus::Disconnected {
            return Some("Device disconnected".to_string());
        }
        None
    }

    pub fn views_for_bucket(&self, bucket: LayoutBucket) -> Vec<&'static str> {
        match bucket {
            LayoutBucket::Narrow => vec![
                "header",
                "status",
                "graph",
                "presets",
                "autoeq",
                "advanced",
                "diagnostics",
            ],
            LayoutBucket::Medium => vec![
                "header",
                "status",
                "graph",
                "autoeq+presets",
                "advanced",
                "diagnostics",
            ],
            LayoutBucket::Wide => vec!["header+status", "left:graph+advanced", "right:tools"],
        }
    }

    fn view_narrow(&self) -> Element<'_, Message> {
        scrollable(
            column![
                views::graph_panel::view_graph(self),
                views::presets_preamp::view_presets_and_preamp(
                    self,
                    views::presets_preamp::PresetsLayout::Narrow,
                ),
                views::autoeq::view_autoeq(self),
                views::bands::view_bands(self),
                views::diagnostics::view_diagnostics_section(self),
            ]
            .spacing(SPACE_16)
            .width(Length::Fill)
        )
        .into()
    }

    fn view_medium(&self) -> Element<'_, Message> {
        let tools_row = row![
            container(views::presets_preamp::view_presets_and_preamp(
                self,
                views::presets_preamp::PresetsLayout::Medium,
            ))
            .width(Length::FillPortion(1))
            .height(Length::Fill),
            container(views::autoeq::view_autoeq(self))
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        ]
        .spacing(SPACE_16)
        .align_y(iced::Alignment::Start)
        .width(Length::Fill);

        scrollable(
            column![
                views::graph_panel::view_graph(self),
                tools_row,
                views::bands::view_bands(self),
                views::diagnostics::view_diagnostics_section(self),
            ]
            .spacing(SPACE_16)
            .width(Length::Fill)
        )
        .into()
    }

    fn view_wide(&self) -> Element<'_, Message> {
        let left_content = column![
            views::graph_panel::view_graph_fill(self),
            views::bands::view_bands(self),
        ]
        .spacing(SPACE_8)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: 0.0, right: SPACE_16, bottom: SPACE_8, left: SPACE_16 });

        let right_sidebar = container(
            scrollable(
                column![
                    views::presets_preamp::view_presets_and_preamp(
                        self,
                        views::presets_preamp::PresetsLayout::Narrow,
                    ),
                    views::autoeq::view_autoeq(self),
                    views::diagnostics::view_diagnostics_section(self),
                ]
                .spacing(SPACE_16)
                .padding(Padding { top: 0.0, right: SPACE_16, bottom: SPACE_16, left: 0.0 })
            )
            .height(Length::Fill)
        )
        .width(Length::Fixed(crate::ui::tokens::SIDEBAR_WIDTH));

        row![left_content, right_sidebar]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn with_modal_overlay<'a>(&self, main_view: Element<'a, Message>) -> Element<'a, Message> {
        if let Some(dialog) = match self.editor_state.pending_confirm {
            ConfirmAction::ResetFilters => Some(views::confirm_dialog::view_confirm_dialog(
                "Reset Filters?",
                "This will reset all 10 bands to default values and set global gain to 0.",
                "Reset",
                Message::ConfirmResetFilters,
            )),
            ConfirmAction::DeleteProfile => Some(views::confirm_dialog::view_confirm_dialog(
                "Delete Profile?",
                "Are you sure you want to delete this profile? This cannot be undone.",
                "Delete",
                Message::ConfirmDeleteProfile,
            )),
            ConfirmAction::None => None,
        } {
            iced::widget::stack![
                main_view,
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Color { a: 0.8, ..crate::ui::theme::TOKYO_NIGHT_BG_DARK }.into()),
                        ..Default::default()
                    })
            ]
            .into()
        } else {
            main_view
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = responsive(move |size| {
            let bucket = layout_bucket_for_width(size.width);
            match bucket {
                LayoutBucket::Narrow => container(self.view_narrow())
                    .padding(SPACE_16)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                LayoutBucket::Medium => container(self.view_medium())
                    .padding(SPACE_24)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                LayoutBucket::Wide => self.view_wide(),
            }
        });

        let main_view = column![
            views::header::view_header(self),
            views::status_banner::view_status_banner(self),
            content,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        self.with_modal_overlay(main_view)
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::pin::Pin;
        use std::time::Duration;
        async fn tick() -> Message {
            Message::Tick(std::time::Instant::now())
        }
        time::repeat(|| Pin::from(Box::pin(tick())), Duration::from_secs(2))
    }
}

pub fn run() -> iced::Result {
    iced::application(MainWindow::new, MainWindow::update, MainWindow::view)
        .title(MainWindow::title)
        .subscription(MainWindow::subscription)
        .theme(MainWindow::app_theme)
        .run()
}
