use crate::autoeq;
use crate::diagnostics::{DiagnosticEvent, DiagnosticsStore, LogLevel, Source};
use crate::error::ErrorKind;
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use crate::models::{
    snap_freq_to_iso, snap_q_to_iso, ConnectionResult, Device, Filter, OperationResult, PEQData,
    MAX_BAND_GAIN, MAX_FREQ, MAX_GLOBAL_GAIN, MAX_Q, MIN_BAND_GAIN, MIN_FREQ, MIN_GLOBAL_GAIN,
    MIN_Q,
};
use crate::ui::graph::EqGraph;
use crate::ui::messages::{Message, StatusMessage, StatusSeverity};
use crate::ui::state::{
    ConfirmAction, ConnectionStatus, DisconnectReason, EditorState, InputBuffer, MainWindow,
    OperationLock,
};
use crate::ui::theme::{
    self, TOKYO_NIGHT_BG, TOKYO_NIGHT_BLUE, TOKYO_NIGHT_ERROR, TOKYO_NIGHT_FG, TOKYO_NIGHT_GREEN,
    TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_RED, TOKYO_NIGHT_SUCCESS,
    TOKYO_NIGHT_WARNING, TOKYO_NIGHT_YELLOW,
};
use crate::ui::tokens::{WINDOW_MEDIUM_MAX, WINDOW_NARROW_MAX};
use iced::{
    clipboard,
    widget::{
        button, canvas, checkbox, column, container, pick_list, responsive, row, scrollable,
        slider, text, text_input,
    },
    Background, Border, Element, Length, Subscription, Task,
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

// COSMIC 8px grid spacing system
const SPACE_4: f32 = 4.0;
const SPACE_8: f32 = 8.0;
const SPACE_12: f32 = 12.0;
const SPACE_16: f32 = 16.0;
const SPACE_24: f32 = 24.0;
const SPACE_2: f32 = 2.0;

// COSMIC typography scale
const TYPE_DISPLAY: f32 = 28.0;
const TYPE_TITLE: f32 = 20.0;
const TYPE_BODY: f32 = 16.0;
const TYPE_LABEL: f32 = 14.0;
const TYPE_CAPTION: f32 = 12.0;
const BUTTON_VERTICAL_PADDING: f32 = 10.0;
const BUTTON_HORIZONTAL_PADDING: f32 = 16.0;

fn action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(text(label).size(TYPE_LABEL))
        .padding([BUTTON_VERTICAL_PADDING, BUTTON_HORIZONTAL_PADDING])
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
                            error: Some("Worker closed".into()),
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
                            error: Some("Worker closed".into()),
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
                            error: Some("Worker closed".into()),
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
                    let err = result.error.unwrap_or_else(|| "Unknown".into());
                    let user_error = match ErrorKind::from_string(&err) {
                        ErrorKind::PolkitAuthRequired => "Authentication required to access USB DAC on Linux. Approve the polkit prompt and retry.".to_string(),
                        _ => err.clone(),
                    };
                    self.connection_status = ConnectionStatus::Error(user_error.clone());
                    self.connected_device = None;
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Connect failed: {}", err),
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
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
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
                    }
                    Task::none()
                } else if let Some(err) = result.error {
                    if err.contains("Not connected") || err.contains("not found") {
                        self.connection_status = ConnectionStatus::Disconnected;
                    } else {
                        self.connection_status = ConnectionStatus::Error(err.clone());
                    }
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Pull failed: {}", err),
                    ));
                    self.set_status(format!("Pull failed: {}", err), StatusSeverity::Error)
                } else {
                    Task::none()
                }
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
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
                    }
                    Task::none()
                } else if let Some(err) = result.error {
                    if err.contains("Not connected") || err.contains("not found") {
                        self.connection_status = ConnectionStatus::Disconnected;
                    } else {
                        self.connection_status = ConnectionStatus::Error(err.clone());
                    }
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Push failed: {}", err),
                    ));
                    self.set_status(format!("Push failed: {}", err), StatusSeverity::Error)
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
                let _write_task: iced::Task<()> = clipboard::write(output);
                self.set_status("Exported to clipboard", StatusSeverity::Success)
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
                let _ = clipboard::write::<()>(output);
                self.set_status("Diagnostics copied", StatusSeverity::Info)
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
                "autoeq",
                "presets",
                "graph",
                "advanced",
                "diagnostics",
            ],
            LayoutBucket::Medium => vec![
                "header",
                "status",
                "autoeq",
                "presets",
                "graph",
                "advanced",
                "diagnostics",
            ],
            LayoutBucket::Wide => vec![
                "header",
                "status",
                "autoeq",
                "presets",
                "graph",
                "advanced",
                "diagnostics",
            ],
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = responsive(move |size| {
            let bucket = layout_bucket_for_width(size.width);
            let (padding, spacing) = if matches!(bucket, LayoutBucket::Narrow) {
                (SPACE_16, SPACE_12)
            } else if matches!(bucket, LayoutBucket::Medium) {
                (SPACE_24, SPACE_16)
            } else {
                (SPACE_24, SPACE_16)
            };

            let _is_wide = matches!(bucket, LayoutBucket::Wide);
            let layout = column![
                self.view_header(),
                self.view_status_banner(),
                self.view_autoeq(),
                self.view_presets_and_preamp(),
                self.view_graph(),
                self.view_advanced_filters_section(),
                self.view_diagnostics_section(),
            ]
            .spacing(spacing);

            container(layout.padding(padding)).into()
        });

        let main_view = container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into();

        if let Some(dialog) = match self.editor_state.pending_confirm {
            ConfirmAction::ResetFilters => Some(self.view_confirm_dialog(
                "Reset Filters?",
                "This will reset all 10 bands to default values and set global gain to 0.",
                "Reset",
                Message::ConfirmResetFilters,
            )),
            ConfirmAction::DeleteProfile => Some(self.view_confirm_dialog(
                "Delete Profile?",
                "Are you sure you want to delete this profile? This cannot be undone.",
                "Delete",
                Message::ConfirmDeleteProfile,
            )),
            ConfirmAction::None => None,
        } {
            container(column![
                container(main_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dialog,
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            main_view
        }
    }

    fn view_confirm_dialog<'a>(
        &self,
        title: &'a str,
        message: &'a str,
        confirm_label: &'a str,
        confirm_msg: Message,
    ) -> Element<'a, Message> {
        container(
            column![
                text(title).size(TYPE_TITLE).color(TOKYO_NIGHT_FG),
                text(message).size(TYPE_LABEL).color(TOKYO_NIGHT_MUTED),
                row![
                    action_button("Cancel")
                        .on_press(Message::DismissConfirmDialog)
                        .style(theme::pill_secondary_button),
                    action_button(confirm_label)
                        .on_press(confirm_msg)
                        .style(theme::pill_danger_button),
                ]
                .spacing(SPACE_12),
            ]
            .spacing(SPACE_12)
            .padding(SPACE_16),
        )
        .style(theme::card_style)
        .width(Length::Fixed(400.0))
        .center_x(Length::Fill)
        .into()
    }

    fn view_status_banner(&self) -> Element<'_, Message> {
        if let Some(msg) = &self.editor_state.status_message {
            let color = match msg.severity {
                StatusSeverity::Info => TOKYO_NIGHT_BLUE,
                StatusSeverity::Success => TOKYO_NIGHT_GREEN,
                StatusSeverity::Warning => TOKYO_NIGHT_YELLOW,
                StatusSeverity::Error => TOKYO_NIGHT_RED,
            };

            container(
                row![
                    text(&msg.content).size(TYPE_BODY).color(TOKYO_NIGHT_BG),
                    container(text("")).width(Length::Fill),
                    button(text("×").size(14))
                        .on_press(Message::ClearStatusMessage)
                        .style(theme::pill_text_button)
                ]
                .spacing(SPACE_16)
                .align_y(iced::Alignment::Center),
            )
            .padding([SPACE_8, SPACE_16])
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        } else {
            column![].into()
        }
    }

    fn view_header(&self) -> Element<'_, Message> {
        let is_busy = self.operation_lock.is_pulling
            || self.operation_lock.is_pushing
            || self.operation_lock.is_connecting
            || self.operation_lock.is_disconnecting;

        let status_text = match &self.connection_status {
            ConnectionStatus::Disconnected => match self.disconnect_reason {
                DisconnectReason::None => text("Disconnected").color(TOKYO_NIGHT_MUTED),
                DisconnectReason::Manual => text("Disconnected (by user)").color(TOKYO_NIGHT_MUTED),
                DisconnectReason::DeviceLost => {
                    text("Disconnected (device unplugged)").color(TOKYO_NIGHT_WARNING)
                }
                DisconnectReason::Error(ref e) => {
                    text(format!("Error: {}", e)).color(TOKYO_NIGHT_ERROR)
                }
            },
            ConnectionStatus::Connecting => text("Connecting...").color(TOKYO_NIGHT_YELLOW),
            ConnectionStatus::Connected => text("Connected").color(TOKYO_NIGHT_SUCCESS),
            ConnectionStatus::Error(e) => text(format!("Error: {}", e)).color(TOKYO_NIGHT_ERROR),
        };

        let device_info_text = if let Some(ref dev) = self.connected_device {
            let device_type = Device::from_vid_pid(dev.vendor_id, dev.product_id);
            let name = device_type.name();
            let vid_pid = format!("VID:{:04X} PID:{:04X}", dev.vendor_id, dev.product_id);
            let mfr = dev
                .manufacturer
                .as_deref()
                .map(|m| format!(" ({})", m))
                .unwrap_or_default();
            let row_el: Element<'_, Message> = column![
                text(format!("Device: {}", name))
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_PRIMARY),
                text(format!("{}{}", vid_pid, mfr))
                    .size(TYPE_CAPTION)
                    .color(TOKYO_NIGHT_MUTED),
            ]
            .spacing(SPACE_4)
            .into();
            row_el
        } else {
            text("No device connected")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED)
                .into()
        };

        let btn_row = row![
            if !is_busy
                && (self.connection_status == ConnectionStatus::Disconnected
                    || matches!(&self.connection_status, ConnectionStatus::Error(_)))
            {
                action_button("Connect")
                    .on_press(Message::ConnectPressed)
                    .style(theme::pill_primary_button)
            } else {
                action_button("Connect").style(theme::pill_primary_button)
            },
            if !is_busy && self.connection_status == ConnectionStatus::Connected {
                action_button("Disconnect")
                    .on_press(Message::DisconnectPressed)
                    .style(theme::pill_secondary_button)
            } else {
                action_button("Disconnect").style(theme::pill_secondary_button)
            },
            if !is_busy && self.connection_status == ConnectionStatus::Connected {
                action_button("Read Device")
                    .on_press(Message::PullPressed)
                    .style(theme::pill_secondary_button)
            } else {
                action_button("Read Device").style(theme::pill_secondary_button)
            },
            if !is_busy && self.connection_status == ConnectionStatus::Connected {
                action_button("Write Device")
                    .on_press(Message::PushPressed)
                    .style(theme::pill_primary_button)
            } else {
                action_button("Write Device").style(theme::pill_primary_button)
            },
        ]
        .spacing(SPACE_8);

        let loading_indicator = if is_busy {
            row![
                text("Processing...")
                    .size(TYPE_CAPTION)
                    .color(TOKYO_NIGHT_BLUE),
                // We could add a spinner here if we had an icon font
            ]
            .spacing(SPACE_8)
            .align_y(iced::Alignment::Center)
        } else {
            row![]
        };

        container(
            row![
                column![
                    text("Frost-Tune")
                        .size(TYPE_DISPLAY)
                        .color(TOKYO_NIGHT_PRIMARY),
                    device_info_text,
                    text("Workflow: Connect → Read Device → Edit → Write Device")
                        .size(TYPE_LABEL)
                        .color(TOKYO_NIGHT_FG),
                    row![status_text.size(TYPE_BODY), loading_indicator].spacing(SPACE_16),
                ]
                .spacing(SPACE_4),
                container(text("")).width(Length::Fill),
                btn_row,
            ]
            .align_y(iced::Alignment::Center),
        )
        .padding(SPACE_12)
        .style(theme::header_card_style)
        .into()
    }

    fn view_presets_and_preamp(&self) -> Element<'_, Message> {
        let is_busy = self.operation_lock.is_pulling || self.operation_lock.is_pushing;

        let preset_names: Vec<String> = self
            .editor_state
            .profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();

        let preset_row = row![
            text("Presets:").size(TYPE_BODY).color(TOKYO_NIGHT_MUTED),
            pick_list(
                preset_names,
                self.editor_state.selected_profile_name.clone(),
                Message::ProfileSelected,
            )
            .placeholder("Select Preset")
            .style(theme::m3_input_pick_list)
            .width(Length::FillPortion(2)),
            text_input("New Name...", &self.editor_state.new_profile_name)
                .on_input(Message::ProfileNameInput)
                .style(theme::m3_filled_input)
                .width(Length::FillPortion(1)),
            action_button("Reset")
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::ResetFiltersPressed)
                })
                .style(theme::pill_secondary_button),
            action_button("Save")
                .on_press(Message::SaveProfilePressed)
                .style(theme::pill_primary_button),
            if !is_busy && self.editor_state.selected_profile_name.is_some() {
                action_button("Delete")
                    .on_press(Message::DeleteProfilePressed)
                    .style(theme::pill_danger_button)
            } else {
                action_button("Delete").style(theme::pill_danger_button)
            },
        ]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center);

        let preamp_row = row![
            text("Preamp:").size(TYPE_BODY).color(TOKYO_NIGHT_MUTED),
            slider(
                MIN_GLOBAL_GAIN as f64..=MAX_GLOBAL_GAIN as f64,
                self.editor_state.global_gain as f64,
                |v| Message::GlobalGainChanged(v as i8)
            )
            .width(Length::Fill),
            text(format!("{} dB", self.editor_state.global_gain))
                .size(TYPE_BODY)
                .width(Length::Fixed(50.0))
                .color(TOKYO_NIGHT_PRIMARY),
        ]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center);

        container(column![preset_row, preamp_row,].spacing(SPACE_16))
            .padding(SPACE_16)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
    }

    fn view_graph(&self) -> Element<'_, Message> {
        responsive(move |size| {
            let height = if size.width < 1000.0 {
                260.0
            } else if size.width < 1280.0 {
                300.0
            } else {
                340.0
            };

            container(
                canvas(EqGraph::new(
                    &self.editor_state.filters,
                    self.editor_state.global_gain,
                ))
                .width(Length::Fill)
                .height(Length::Fixed(height)),
            )
            .padding(SPACE_12)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
        })
        .into()
    }

    fn view_bands(&self) -> Element<'_, Message> {
        let is_busy = self.operation_lock.is_pulling || self.operation_lock.is_pushing;

        let busy_notice: Element<Message> = if is_busy {
            container(
                text("Device sync in progress... controls temporarily locked")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_WARNING),
            )
            .padding(SPACE_12)
            .into()
        } else {
            text("").into()
        };

        let band_list: Vec<Element<Message>> = self
            .editor_state
            .filters
            .iter()
            .enumerate()
            .map(|(i, band)| {
                let freq_error = self.editor_state.input_buffer.get_freq_error(i);
                let gain_error = self.editor_state.input_buffer.get_gain_error(i);
                let q_error = self.editor_state.input_buffer.get_q_error(i);

                let freq_error_display = if let Some(err) = freq_error {
                    text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
                } else {
                    text("")
                };
                let gain_error_display = if let Some(err) = gain_error {
                    text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
                } else {
                    text("")
                };
                let q_error_display = if let Some(err) = q_error {
                    text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
                } else {
                    text("")
                };

                column![
                    row![
                        text(format!("{}", i + 1))
                            .size(TYPE_BODY)
                            .width(Length::Fixed(20.0)),
                        pick_list(
                            &[
                                crate::models::FilterType::LowShelf,
                                crate::models::FilterType::Peak,
                                crate::models::FilterType::HighShelf
                            ][..],
                            Some(band.filter_type),
                            move |t| Message::BandTypeChanged(i, t),
                        )
                        .width(Length::Fixed(110.0))
                        .style(theme::m3_input_pick_list)
                        .text_size(12),
                        row![text_input(
                            "",
                            self.editor_state
                                .input_buffer
                                .get_freq(i)
                                .as_deref()
                                .unwrap_or(&format!("{}", band.freq))
                        )
                        .on_input(move |s| Message::BandFreqInput(i, s))
                        .on_submit(Message::BandFreqInputCommit(i))
                        .style(theme::m3_outlined_input)
                        .width(Length::Fixed(80.0))
                        .size(TYPE_LABEL),]
                        .spacing(SPACE_4)
                        .align_y(iced::Alignment::Center)
                        .width(Length::FillPortion(2)),
                        row![
                            slider(MIN_BAND_GAIN..=MAX_BAND_GAIN, band.gain, move |v| {
                                Message::BandGainChanged(i, v)
                            })
                            .step(0.1)
                            .width(Length::Fill),
                            text_input(
                                "",
                                self.editor_state
                                    .input_buffer
                                    .get_gain(i)
                                    .as_deref()
                                    .unwrap_or(&format!("{:.2}", band.gain))
                            )
                            .on_input(move |s| Message::BandGainInput(i, s))
                            .on_submit(Message::BandGainInputCommit(i))
                            .style(theme::m3_outlined_input)
                            .width(Length::Fixed(60.0))
                            .size(TYPE_LABEL),
                        ]
                        .spacing(SPACE_4)
                        .align_y(iced::Alignment::Center)
                        .width(Length::FillPortion(4)),
                        row![text_input(
                            "",
                            self.editor_state
                                .input_buffer
                                .get_q(i)
                                .as_deref()
                                .unwrap_or(&format!("{:.2}", band.q))
                        )
                        .on_input(move |s| Message::BandQInput(i, s))
                        .on_submit(Message::BandQInputCommit(i))
                        .style(theme::m3_outlined_input)
                        .width(Length::Fixed(60.0))
                        .size(TYPE_LABEL),]
                        .spacing(SPACE_4)
                        .align_y(iced::Alignment::Center)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(SPACE_4)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("").width(Length::Fixed(20.0)),
                        text("").width(Length::Fixed(110.0)),
                        freq_error_display.width(Length::FillPortion(2)),
                        gain_error_display.width(Length::FillPortion(4)),
                        q_error_display.width(Length::FillPortion(1)),
                    ]
                    .spacing(SPACE_4),
                ]
                .spacing(SPACE_2)
                .into()
            })
            .collect();

        let header = row![
            text("#")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED)
                .width(Length::Fixed(20.0)),
            text("Type")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED)
                .width(Length::Fixed(110.0)),
            text("Frequency (Hz)")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED)
                .width(Length::FillPortion(2)),
            row![
                container(text("Gain (dB)").size(TYPE_LABEL).color(TOKYO_NIGHT_MUTED))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                container(text("")).width(Length::Fixed(60.0)),
            ]
            .width(Length::FillPortion(4)),
            text("Q")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED)
                .width(Length::FillPortion(1)),
        ]
        .spacing(SPACE_4)
        .align_y(iced::Alignment::Center);

        container(
            container(column![
                busy_notice,
                header,
                scrollable(column(band_list).spacing(SPACE_8))
            ])
            .max_width(1080),
        )
        .padding([SPACE_12, SPACE_8])
        .style(theme::card_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
    }

    fn view_advanced_filters_section(&self) -> Element<'_, Message> {
        let expanded = self.editor_state.advanced_filters_expanded;
        let toggle_text = if expanded {
            "Hide advanced filters"
        } else {
            "Show advanced filters"
        };

        let heading = row![
            column![
                text("Advanced filter controls")
                    .size(TYPE_TITLE)
                    .color(TOKYO_NIGHT_PRIMARY),
                text("Manual PEQ editing for advanced users")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_MUTED),
            ]
            .spacing(SPACE_4),
            container(text("")).width(Length::Fill),
            action_button(toggle_text)
                .on_press(Message::ToggleAdvancedFilters(!expanded))
                .style(theme::pill_secondary_button),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(SPACE_12);

        let body: Element<Message> = if expanded {
            self.view_bands()
        } else {
            container(
                text("AutoEQ import/export is recommended for most users.")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_MUTED),
            )
            .padding([SPACE_12, SPACE_8])
            .into()
        };

        container(column![heading, body].spacing(SPACE_12))
            .padding(SPACE_16)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
    }

    fn view_autoeq(&self) -> Element<'_, Message> {
        let is_busy = self.operation_lock.is_pulling || self.operation_lock.is_pushing;

        container(
            column![
                text("AutoEQ").size(TYPE_TITLE).color(TOKYO_NIGHT_PRIMARY),
                text("Import/export standard parametric EQ text files (AutoEQ format).")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_MUTED),
                row![
                    action_button("Import Clipboard")
                        .on_press_maybe(if is_busy {
                            None
                        } else {
                            Some(Message::ImportFromClipboard)
                        })
                        .style(theme::pill_secondary_button),
                    action_button("Import File")
                        .on_press_maybe(if is_busy {
                            None
                        } else {
                            Some(Message::ImportFromFilePressed)
                        })
                        .style(theme::pill_secondary_button),
                ]
                .spacing(SPACE_8),
                row![
                    action_button("Export Clipboard")
                        .on_press_maybe(if is_busy {
                            None
                        } else {
                            Some(Message::ExportAutoEQPressed)
                        })
                        .style(theme::pill_secondary_button),
                    action_button("Export File")
                        .on_press_maybe(if is_busy {
                            None
                        } else {
                            Some(Message::ExportToFilePressed)
                        })
                        .style(theme::pill_secondary_button),
                ]
                .spacing(SPACE_8),
            ]
            .spacing(SPACE_12),
        )
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::FillPortion(1))
        .into()
    }

    fn view_diagnostics(&self) -> Element<'_, Message> {
        let has_events = self.diagnostics.count() > 0;
        let diag_events: Vec<Element<Message>> = self
            .diagnostics
            .events()
            .filter(|e| !self.editor_state.diagnostics_errors_only || e.level == LogLevel::Error)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(20)
            .map(|e| {
                let c = match e.level {
                    LogLevel::Info => TOKYO_NIGHT_MUTED,
                    LogLevel::Warn => TOKYO_NIGHT_WARNING,
                    LogLevel::Error => TOKYO_NIGHT_ERROR,
                };
                text(format!("[{}] {} {}", e.level, e.source, e.message))
                    .size(TYPE_LABEL)
                    .color(c)
                    .into()
            })
            .collect();

        let visible_count = diag_events.len().min(20);
        let logs_height = if visible_count == 0 {
            84.0
        } else if visible_count < 4 {
            92.0
        } else if visible_count < 10 {
            120.0
        } else {
            160.0
        };

        container(
            column![
                row![
                    checkbox(self.editor_state.diagnostics_errors_only)
                        .label("Errors Only")
                        .on_toggle(Message::ToggleDiagnosticsErrorsOnly)
                        .size(14),
                    container(text("")).width(Length::Fill),
                ]
                .align_y(iced::Alignment::Center),
                if diag_events.is_empty() {
                    container(
                        text("No diagnostics events yet. Events will appear during connection, import, and sync operations.")
                            .size(TYPE_LABEL)
                            .color(TOKYO_NIGHT_MUTED),
                    )
                    .height(Length::Fixed(logs_height))
                    .align_y(iced::alignment::Vertical::Center)
                } else {
                    container(scrollable(column(diag_events).spacing(2)).height(Length::Fixed(logs_height)))
                },
                row![
                    action_button("Copy")
                        .on_press(Message::CopyDiagnostics)
                        .style(theme::pill_text_button),
                    action_button("Export")
                        .on_press(Message::ExportDiagnosticsToFile)
                        .style(theme::pill_text_button),
                    action_button("Clear")
                        .on_press(Message::ClearDiagnostics)
                        .style(if has_events {
                            theme::pill_outlined_danger_button
                        } else {
                            theme::pill_text_button
                        }),
                ]
                .spacing(SPACE_8),
            ]
            .spacing(SPACE_12),
        )
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::FillPortion(1))
        .into()
    }

    fn view_diagnostics_section(&self) -> Element<'_, Message> {
        let expanded = self.editor_state.diagnostics_expanded;
        let toggle_text = if expanded {
            "Hide diagnostics"
        } else {
            "Show diagnostics"
        };

        let header = row![
            text("Diagnostics")
                .size(TYPE_TITLE)
                .color(TOKYO_NIGHT_PRIMARY),
            container(text("")).width(Length::Fill),
            action_button(toggle_text)
                .on_press(Message::ToggleDiagnosticsExpanded(!expanded))
                .style(theme::pill_secondary_button),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(SPACE_12);

        let body: Element<Message> = if expanded {
            self.view_diagnostics()
        } else {
            container(
                text("Diagnostics are hidden by default. Expand when troubleshooting.")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_MUTED),
            )
            .padding([SPACE_12, SPACE_8])
            .into()
        };

        container(column![header, body].spacing(SPACE_12))
            .padding(SPACE_16)
            .style(theme::card_style)
            .width(Length::FillPortion(1))
            .into()
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
