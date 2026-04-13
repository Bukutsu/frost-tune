use crate::autoeq;
use crate::diagnostics::{DiagnosticEvent, DiagnosticsStore, LogLevel, Source};
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use crate::models::{
    ConnectionResult, Filter, OperationResult, PEQData, MAX_BAND_GAIN, MAX_GLOBAL_GAIN,
    MIN_BAND_GAIN, MIN_GLOBAL_GAIN,
};
use crate::ui::graph::EqGraph;
use crate::ui::theme::{
    self, TOKYO_NIGHT_ERROR, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_SUCCESS,
    TOKYO_NIGHT_TEXT, TOKYO_NIGHT_WARNING,
};
use iced::{
    clipboard,
    widget::{
        button, canvas, checkbox, column, container, pick_list, responsive, row, scrollable,
        slider, text, text_input, toggler,
    },
    Color, Element, Length, Subscription, Task,
};
use std::sync::Arc;

const SPACE_XS: f32 = 8.0;
const SPACE_SM: f32 = 12.0;
const SPACE_MD: f32 = 16.0;
const SPACE_LG: f32 = 24.0;

const TYPE_DISPLAY: f32 = 28.0;
const TYPE_TITLE: f32 = 16.0;
const TYPE_BODY: f32 = 14.0;
const TYPE_LABEL: f32 = 12.0;

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

#[derive(Debug, Clone)]
pub enum Message {
    ConnectPressed,
    DisconnectPressed,
    PullPressed,
    PushPressed,
    WorkerConnected(ConnectionResult),
    WorkerDisconnected(OperationResult),
    WorkerPulled(OperationResult),
    WorkerPushed(OperationResult),
    WorkerStatus(WorkerStatus),
    Tick(std::time::Instant),
    BandEnabledChanged(usize, bool),
    BandGainChanged(usize, f64),
    BandFreqChanged(usize, u16),
    BandQChanged(usize, f64),
    BandTypeChanged(usize, crate::models::FilterType),
    BandGainInput(usize, String),
    BandFreqInput(usize, String),
    BandQInput(usize, String),
    BandFreqSliderChanged(usize, f64),
    GlobalGainChanged(i8),
    ImportFromClipboard,
    ImportClipboardReceived(String),
    ImportClipboardFailed(String),
    ExportAutoEQPressed,
    ExportComplete,
    CopyDiagnostics,
    ClearDiagnostics,
    ToggleDiagnosticsErrorsOnly(bool),
    ExportDiagnosticsToFile,
    DiagnosticsExported(String),
    ProfilesLoaded(Vec<crate::storage::Profile>),
    ProfileSelected(String),
    ProfileNameInput(String),
    SaveProfilePressed,
    DeleteProfilePressed,
    ImportFromFilePressed,
    ExportToFilePressed,
    FileImported(Result<String, String>),
    FileExported(Result<String, String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditorState {
    pub filters: Vec<Filter>,
    pub global_gain: i8,
    pub autoeq_message: Option<String>,
    pub diagnostics_errors_only: bool,
    pub profiles: Vec<crate::storage::Profile>,
    pub selected_profile_name: Option<String>,
    pub new_profile_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct OperationLock {
    pub is_pulling: bool,
    pub is_pushing: bool,
    pub is_connecting: bool,
}

#[derive(Default)]
pub struct MainWindow {
    connection_status: ConnectionStatus,
    editor_state: EditorState,
    operation_lock: OperationLock,
    worker: Option<Arc<UsbWorker>>,
    diagnostics: DiagnosticsStore,
}

impl MainWindow {
    fn new() -> (Self, Task<Message>) {
        let worker = Arc::new(UsbWorker::new());
        let default_filters: Vec<Filter> =
            (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let window = MainWindow {
            connection_status: ConnectionStatus::Disconnected,
            editor_state: EditorState {
                filters: default_filters.clone(),
                global_gain: 0,
                autoeq_message: None,
                diagnostics_errors_only: false,
                profiles: Vec::new(),
                selected_profile_name: None,
                new_profile_name: String::new(),
            },
            operation_lock: OperationLock::default(),
            worker: Some(worker),
            diagnostics: DiagnosticsStore::default(),
        };
        let load_task = Task::perform(
            async move { crate::storage::load_all_profiles().unwrap_or_default() },
            Message::ProfilesLoaded,
        );
        (window, load_task)
    }

    fn title(&self) -> String {
        "Frost-Tune".into()
    }

    fn app_theme(_state: &Self) -> iced::Theme {
        theme::theme()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
                Task::perform(
                    async move {
                        let rx = worker.connect();
                        rx.recv().unwrap_or(ConnectionResult {
                            success: false,
                            device: None,
                            error: Some("Worker closed".into()),
                        })
                    },
                    Message::WorkerConnected,
                )
            }
            Message::DisconnectPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Disconnect pressed",
                ));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(
                    async move {
                        let rx = worker.disconnect();
                        rx.recv().unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some("Worker closed".into()),
                        })
                    },
                    Message::WorkerDisconnected,
                )
            }
            Message::PullPressed => {
                if self.worker.is_none()
                    || self.operation_lock.is_pulling
                    || self.operation_lock.is_pushing
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
                Task::perform(
                    async move {
                        let rx = worker.pull_peq();
                        rx.recv().unwrap_or(OperationResult {
                            success: false,
                            data: None,
                            error: Some("Worker closed".into()),
                        })
                    },
                    Message::WorkerPulled,
                )
            }
            Message::PushPressed => {
                if self.worker.is_none()
                    || self.operation_lock.is_pulling
                    || self.operation_lock.is_pushing
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
                Task::perform(
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
                )
            }
            Message::WorkerConnected(result) => {
                self.operation_lock.is_connecting = false;
                if result.success {
                    self.connection_status = ConnectionStatus::Connected;
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Connected successfully",
                    ));
                } else {
                    let err = result.error.unwrap_or_else(|| "Unknown".into());
                    self.connection_status = ConnectionStatus::Error(err.clone());
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::Worker,
                        format!("Connect failed: {}", err),
                    ));
                }
                Task::none()
            }
            Message::WorkerDisconnected(_) => {
                self.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::Worker,
                    "Disconnected",
                ));
                self.connection_status = ConnectionStatus::Disconnected;
                Task::none()
            }
            Message::WorkerPulled(result) => {
                self.operation_lock.is_pulling = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
                            self.editor_state.filters = peq.filters;
                            self.editor_state.global_gain = peq.global_gain;
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::Worker,
                                "Pull successful",
                            ));
                        }
                    }
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
                }
                Task::none()
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
                            self.editor_state.filters = peq.filters;
                            self.editor_state.global_gain = peq.global_gain;
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::Worker,
                                "Push successful",
                            ));
                        }
                    }
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
                }
                Task::none()
            }
            Message::WorkerStatus(status) => {
                if status.connected && self.connection_status != ConnectionStatus::Connected {
                    self.connection_status = ConnectionStatus::Connected;
                    log::info!("Device connected");
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::Worker,
                        "Device connected (poll)",
                    ));
                } else if !status.connected && self.connection_status == ConnectionStatus::Connected
                {
                    self.connection_status = ConnectionStatus::Disconnected;
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
                        })
                    },
                    Message::WorkerStatus,
                )
            }
            Message::BandEnabledChanged(index, enabled) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.enabled = enabled;
                }
                Task::none()
            }
            Message::BandGainChanged(index, gain) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.gain = gain;
                    band.enabled = true;
                    band.clamp();
                }
                Task::none()
            }
            Message::BandFreqChanged(index, freq) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.freq = freq;
                    band.clamp();
                }
                Task::none()
            }
            Message::BandQChanged(index, q) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    band.q = q;
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
            Message::BandGainInput(index, s) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    let parsed = s.trim().parse::<f64>();
                    if let Ok(v) = parsed {
                        band.gain = v;
                        band.enabled = true;
                        band.clamp();
                    }
                }
                Task::none()
            }
            Message::BandFreqInput(index, s) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    if let Some(v) = parse_freq_string(&s) {
                        band.freq = v;
                        band.clamp();
                    }
                }
                Task::none()
            }
            Message::BandQInput(index, s) => {
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    if let Ok(v) = s.trim().parse::<f64>() {
                        band.q = v;
                        band.clamp();
                    }
                }
                Task::none()
            }
            Message::BandFreqSliderChanged(index, v) => {
                // v is log10(freq) - convert back
                if let Some(band) = self.editor_state.filters.get_mut(index) {
                    let hz = 10f64.powf(v).round() as u16;
                    band.freq = hz;
                    band.clamp();
                }
                Task::none()
            }
            Message::GlobalGainChanged(gain) => {
                self.editor_state.global_gain = gain;
                Task::none()
            }
            Message::ExportAutoEQPressed => {
                let peq = PEQData {
                    filters: self.editor_state.filters.clone(),
                    global_gain: self.editor_state.global_gain,
                };
                let output = autoeq::peq_to_autoeq(&peq);
                self.editor_state.autoeq_message = Some("Exported to clipboard".into());
                let _write_task: iced::Task<()> = clipboard::write(output);
                Task::none()
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
            Message::ImportClipboardReceived(text) => {
                match autoeq::parse_autoeq_text(&text) {
                    Ok(peq) => {
                        let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
                        self.editor_state.filters = peq.filters;
                        self.editor_state.global_gain = peq.global_gain;
                        self.editor_state.autoeq_message =
                            Some(format!("Imported {} filters from clipboard", enabled_count));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::AutoEQ,
                            format!("Import successful: {} filters", enabled_count),
                        ));
                    }
                    Err(e) => {
                        self.editor_state.autoeq_message = Some(format!("Error: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::AutoEQ,
                            format!("Import failed: {}", e),
                        ));
                    }
                }
                Task::none()
            }
            Message::ImportClipboardFailed(msg) => {
                self.editor_state.autoeq_message = Some(msg.clone());
                self.diagnostics
                    .push(DiagnosticEvent::new(LogLevel::Error, Source::AutoEQ, msg));
                Task::none()
            }
            Message::CopyDiagnostics => {
                let conn_str = format!("{:?}", self.connection_status);
                let output =
                    crate::diagnostics::format_diagnostics(&self.diagnostics, "0.1.0", &conn_str);
                self.editor_state.autoeq_message = Some("Diagnostics copied to clipboard".into());
                clipboard::write::<()>(output).map(|_| Message::ExportComplete)
            }
            Message::ClearDiagnostics => {
                self.diagnostics.clear();
                self.editor_state.autoeq_message = Some("Diagnostics cleared".into());
                Task::none()
            }
            Message::ToggleDiagnosticsErrorsOnly(v) => {
                self.editor_state.diagnostics_errors_only = v;
                Task::none()
            }
            Message::ExportDiagnosticsToFile => {
                let conn_str = format!("{:?}", self.connection_status);
                let output =
                    crate::diagnostics::format_diagnostics(&self.diagnostics, "0.1.0", &conn_str);
                let now = chrono::Local::now();
                let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
                let path = std::path::PathBuf::from(&filename);
                match std::fs::write(&path, output) {
                    Ok(_) => Task::done(Message::DiagnosticsExported(filename)),
                    Err(e) => {
                        self.editor_state.autoeq_message = Some(format!("Export failed: {}", e));
                        Task::none()
                    }
                }
            }
            Message::DiagnosticsExported(name) => {
                self.editor_state.autoeq_message = Some(format!("Saved to {}", name));
                Task::none()
            }
            Message::ProfilesLoaded(profiles) => {
                self.editor_state.profiles = profiles;
                Task::none()
            }
            Message::ProfileSelected(name) => {
                if let Some(profile) = self.editor_state.profiles.iter().find(|p| p.name == name) {
                    self.editor_state.filters = profile.data.filters.clone();
                    self.editor_state.global_gain = profile.data.global_gain;
                    self.editor_state.selected_profile_name = Some(name);
                    self.editor_state.new_profile_name = profile.name.clone();
                    self.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Loaded profile: {}", profile.name),
                    ));
                }
                Task::none()
            }
            Message::ProfileNameInput(name) => {
                self.editor_state.new_profile_name = name;
                Task::none()
            }
            Message::SaveProfilePressed => {
                let name = self.editor_state.new_profile_name.trim().to_string();
                if name.is_empty() {
                    self.editor_state.autoeq_message = Some("Invalid profile name".into());
                    return Task::none();
                }
                let data = PEQData {
                    filters: self.editor_state.filters.clone(),
                    global_gain: self.editor_state.global_gain,
                };
                match crate::storage::save_profile(&name, &data) {
                    Ok(_) => {
                        self.editor_state.autoeq_message = Some(format!("Saved profile: {}", name));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Saved profile: {}", name),
                        ));
                        Task::perform(
                            async move { crate::storage::load_all_profiles().unwrap_or_default() },
                            Message::ProfilesLoaded,
                        )
                    }
                    Err(e) => {
                        self.editor_state.autoeq_message = Some(format!("Failed to save: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Save failed: {}", e),
                        ));
                        Task::none()
                    }
                }
            }
            Message::DeleteProfilePressed => {
                let name = match &self.editor_state.selected_profile_name {
                    Some(n) => n.clone(),
                    None => return Task::none(),
                };
                match crate::storage::delete_profile(&name) {
                    Ok(_) => {
                        self.editor_state.autoeq_message =
                            Some(format!("Deleted profile: {}", name));
                        self.editor_state.selected_profile_name = None;
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Deleted profile: {}", name),
                        ));
                        Task::perform(
                            async move { crate::storage::load_all_profiles().unwrap_or_default() },
                            Message::ProfilesLoaded,
                        )
                    }
                    Err(e) => {
                        self.editor_state.autoeq_message = Some(format!("Failed to delete: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Delete failed: {}", e),
                        ));
                        Task::none()
                    }
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
            Message::FileImported(result) => {
                match result {
                    Ok(text) => match autoeq::parse_autoeq_text(&text) {
                        Ok(peq) => {
                            let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
                            self.editor_state.filters = peq.filters;
                            self.editor_state.global_gain = peq.global_gain;
                            self.editor_state.autoeq_message =
                                Some(format!("Imported {} filters from file", enabled_count));
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::AutoEQ,
                                format!("Import successful from file: {} filters", enabled_count),
                            ));
                        }
                        Err(e) => {
                            self.editor_state.autoeq_message = Some(format!("Error: {}", e));
                            self.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Error,
                                Source::AutoEQ,
                                format!("Import failed: {}", e),
                            ));
                        }
                    },
                    Err(e) if e != "Cancelled" => {
                        self.editor_state.autoeq_message = Some(format!("File error: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("File error: {}", e),
                        ));
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::FileExported(result) => {
                match result {
                    Ok(name) => {
                        self.editor_state.autoeq_message = Some(format!("Exported to {}", name));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Exported to {}", name),
                        ));
                    }
                    Err(e) if e != "Cancelled" => {
                        self.editor_state.autoeq_message = Some(format!("Export error: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Export error: {}", e),
                        ));
                    }
                    _ => {}
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            self.view_header(),
            self.view_presets_and_preamp(),
            self.view_graph(),
            self.view_bands(),
            responsive(move |size| {
                if size.width < 1000.0 {
                    column![self.view_autoeq(), self.view_diagnostics(),]
                        .spacing(SPACE_MD)
                        .into()
                } else {
                    row![self.view_autoeq(), self.view_diagnostics(),]
                        .spacing(SPACE_MD)
                        .into()
                }
            }),
        ]
        .spacing(SPACE_MD)
        .padding(SPACE_LG);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let status_text = match &self.connection_status {
            ConnectionStatus::Disconnected => text("Disconnected").color(TOKYO_NIGHT_MUTED),
            ConnectionStatus::Connecting => {
                text("Connecting...").color(Color::from_rgb(1.0, 0.9, 0.0))
            }
            ConnectionStatus::Connected => text("Connected").color(TOKYO_NIGHT_SUCCESS),
            ConnectionStatus::Error(e) => text(format!("Error: {}", e)).color(TOKYO_NIGHT_ERROR),
        };

        let btn_row = row![
            if self.connection_status == ConnectionStatus::Disconnected
                || matches!(&self.connection_status, ConnectionStatus::Error(_))
            {
                button("Connect").on_press(Message::ConnectPressed)
            } else {
                button("Connect")
            },
            if self.connection_status == ConnectionStatus::Connected {
                button("Disconnect")
                    .on_press(Message::DisconnectPressed)
                    .style(iced::widget::button::danger)
            } else {
                button("Disconnect")
            },
            if self.connection_status == ConnectionStatus::Connected
                && !self.operation_lock.is_pulling
                && !self.operation_lock.is_pushing
            {
                button("Pull").on_press(Message::PullPressed)
            } else {
                button("Pull")
            },
            if self.connection_status == ConnectionStatus::Connected
                && !self.operation_lock.is_pulling
                && !self.operation_lock.is_pushing
            {
                button("Push").on_press(Message::PushPressed)
            } else {
                button("Push")
            },
        ]
        .spacing(SPACE_XS);

        row![
            column![
                text("Frost-Tune")
                    .size(TYPE_DISPLAY)
                    .color(TOKYO_NIGHT_PRIMARY),
                status_text.size(TYPE_BODY),
            ]
            .spacing(4),
            container(text("")).width(Length::Fill),
            btn_row,
        ]
        .align_y(iced::Alignment::Center)
        .into()
    }

    fn view_presets_and_preamp(&self) -> Element<'_, Message> {
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
            .width(Length::Fixed(200.0)),
            text_input("New Name...", &self.editor_state.new_profile_name)
                .on_input(Message::ProfileNameInput)
                .width(Length::Fixed(150.0)),
            button("Save").on_press(Message::SaveProfilePressed),
            if self.editor_state.selected_profile_name.is_some() {
                button("Delete")
                    .on_press(Message::DeleteProfilePressed)
                    .style(iced::widget::button::danger)
            } else {
                button("Delete")
            },
        ]
        .spacing(SPACE_SM)
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
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center);

        container(column![preset_row, preamp_row,].spacing(SPACE_MD))
            .padding(SPACE_MD)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
    }

    fn view_graph(&self) -> Element<'_, Message> {
        container(
            canvas(EqGraph::new(
                &self.editor_state.filters,
                self.editor_state.global_gain,
            ))
            .width(Length::Fill)
            .height(Length::Fixed(220.0)),
        )
        .padding(SPACE_SM)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
    }

    fn view_bands(&self) -> Element<'_, Message> {
        let band_list: Vec<Element<Message>> = self
            .editor_state
            .filters
            .iter()
            .enumerate()
            .map(|(i, band)| {
                row![
                    toggler(band.enabled)
                        .label("")
                        .on_toggle(move |v| Message::BandEnabledChanged(i, v))
                        .size(16),
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
                    .text_size(12),
                    row![
                        text("Freq")
                            .size(TYPE_LABEL)
                            .color(TOKYO_NIGHT_MUTED)
                            .width(Length::Fixed(32.0)),
                        slider(
                            20.0f64.log10()..=20000.0f64.log10(),
                            (band.freq as f64).log10(),
                            move |v| Message::BandFreqSliderChanged(i, v)
                        )
                        .width(Length::Fill),
                        text_input("", &format!("{}", band.freq))
                            .on_input(move |s| Message::BandFreqInput(i, s))
                            .width(Length::Fixed(60.0))
                            .size(TYPE_LABEL),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(3)),
                    row![
                        text("Gain")
                            .size(TYPE_LABEL)
                            .color(TOKYO_NIGHT_MUTED)
                            .width(Length::Fixed(36.0)),
                        slider(MIN_BAND_GAIN..=MAX_BAND_GAIN, band.gain, move |v| {
                            Message::BandGainChanged(i, v)
                        })
                        .width(Length::Fill),
                        text_input("", &format!("{:.1}", band.gain))
                            .on_input(move |s| Message::BandGainInput(i, s))
                            .width(Length::Fixed(50.0))
                            .size(TYPE_LABEL),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(3)),
                    row![
                        text("Q")
                            .size(TYPE_LABEL)
                            .color(TOKYO_NIGHT_MUTED)
                            .width(Length::Fixed(18.0)),
                        slider(0.1..=20.0, band.q, move |v| Message::BandQChanged(i, v))
                            .width(Length::Fill),
                        text_input("", &format!("{:.2}", band.q))
                            .on_input(move |s| Message::BandQInput(i, s))
                            .width(Length::Fixed(50.0))
                            .size(TYPE_LABEL),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(2)),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center)
                .into()
            })
            .collect();

        container(scrollable(column(band_list).spacing(SPACE_XS)))
            .padding(SPACE_SM)
            .style(theme::card_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_autoeq(&self) -> Element<'_, Message> {
        let msg_text = if let Some(ref msg) = self.editor_state.autoeq_message {
            text(msg).size(TYPE_LABEL).color(TOKYO_NIGHT_TEXT)
        } else {
            text("").size(TYPE_LABEL)
        };

        container(
            column![
                text("AutoEQ").size(TYPE_TITLE).color(TOKYO_NIGHT_PRIMARY),
                text("Import/export standard parametric EQ text files (AutoEQ format).")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_MUTED),
                row![
                    button("Import Clipboard").on_press(Message::ImportFromClipboard),
                    button("Import File").on_press(Message::ImportFromFilePressed),
                ]
                .spacing(SPACE_XS),
                row![
                    button("Export Clipboard").on_press(Message::ExportAutoEQPressed),
                    button("Export File").on_press(Message::ExportToFilePressed),
                ]
                .spacing(SPACE_XS),
                msg_text,
            ]
            .spacing(SPACE_SM),
        )
        .padding(SPACE_MD)
        .style(theme::card_style)
        .width(Length::FillPortion(1))
        .into()
    }

    fn view_diagnostics(&self) -> Element<'_, Message> {
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
                    .size(TYPE_LABEL - 1.0)
                    .color(c)
                    .into()
            })
            .collect();

        container(
            column![
                row![
                    text("Diagnostics")
                        .size(TYPE_TITLE)
                        .color(TOKYO_NIGHT_PRIMARY),
                    container(text("")).width(Length::Fill),
                    checkbox(self.editor_state.diagnostics_errors_only)
                        .label("Errors Only")
                        .on_toggle(Message::ToggleDiagnosticsErrorsOnly)
                        .size(14),
                ]
                .align_y(iced::Alignment::Center),
                scrollable(column(diag_events).spacing(2)).height(Length::Fixed(140.0)),
                row![
                    button("Copy").on_press(Message::CopyDiagnostics),
                    button("Export").on_press(Message::ExportDiagnosticsToFile),
                    button("Clear")
                        .on_press(Message::ClearDiagnostics)
                        .style(iced::widget::button::danger),
                ]
                .spacing(SPACE_XS),
            ]
            .spacing(SPACE_SM),
        )
        .padding(SPACE_MD)
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
