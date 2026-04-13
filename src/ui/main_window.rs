use std::sync::Arc;
use iced::{
    Element, Task, Subscription, Theme, clipboard,
    widget::{button, column, text, row, container, scrollable, slider, checkbox},
    Length,
};
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use crate::models::{ConnectionResult, OperationResult, PEQData, Filter, MAX_BAND_GAIN, MIN_BAND_GAIN, MAX_GLOBAL_GAIN, MIN_GLOBAL_GAIN};
use crate::autoeq;
use crate::diagnostics::{DiagnosticsStore, DiagnosticEvent, LogLevel, Source};

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
    GlobalGainChanged(i8),
    ImportFromClipboard,
    ImportClipboardReceived(String),
    ImportClipboardFailed(String),
    ExportAutoEQPressed,
    ExportComplete,
    CopyDiagnostics,
    ClearDiagnostics,
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
        let default_filters: Vec<Filter> = (0..10)
            .map(|i| Filter::enabled(i as u8, false))
            .collect();
        let window = MainWindow {
            connection_status: ConnectionStatus::Disconnected,
            editor_state: EditorState { 
                filters: default_filters.clone(), 
                global_gain: 0,
                autoeq_message: None,
            },
            operation_lock: OperationLock::default(),
            worker: Some(worker),
            diagnostics: DiagnosticsStore::default(),
        };
        (window, Task::none())
    }

    fn title(&self) -> String { "Frost-Tune".into() }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConnectPressed => {
                if self.worker.is_none() { return Task::none(); }
                self.connection_status = ConnectionStatus::Connecting;
                self.operation_lock.is_connecting = true;
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::UI, "Connect pressed"));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.connect();
                    rx.recv().unwrap_or(ConnectionResult { success: false, device: None, error: Some("Worker closed".into()) })
                }, Message::WorkerConnected)
            }
            Message::DisconnectPressed => {
                if self.worker.is_none() { return Task::none(); }
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::UI, "Disconnect pressed"));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.disconnect();
                    rx.recv().unwrap_or(OperationResult { success: false, data: None, error: Some("Worker closed".into()) })
                }, Message::WorkerDisconnected)
            }
            Message::PullPressed => {
                if self.worker.is_none() || self.operation_lock.is_pulling || self.operation_lock.is_pushing { return Task::none(); }
                self.operation_lock.is_pulling = true;
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::UI, "Pull pressed"));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.pull_peq();
                    rx.recv().unwrap_or(OperationResult { success: false, data: None, error: Some("Worker closed".into()) })
                }, Message::WorkerPulled)
            }
            Message::PushPressed => {
                if self.worker.is_none() || self.operation_lock.is_pulling || self.operation_lock.is_pushing { return Task::none(); }
                self.operation_lock.is_pushing = true;
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::UI, "Push pressed"));
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                let filters = self.editor_state.filters.clone();
                let global_gain = self.editor_state.global_gain;
                Task::perform(async move {
                    use crate::models::PushPayload;
                    let payload = PushPayload { filters, global_gain: Some(global_gain) };
                    let rx = worker.push_peq(payload);
                    rx.recv().unwrap_or(OperationResult { success: false, data: None, error: Some("Worker closed".into()) })
                }, Message::WorkerPushed)
            }
            Message::WorkerConnected(result) => {
                self.operation_lock.is_connecting = false;
                if result.success { 
                    self.connection_status = ConnectionStatus::Connected;
                    self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Connected successfully"));
                } else {
                    let err = result.error.unwrap_or_else(|| "Unknown".into());
                    self.connection_status = ConnectionStatus::Error(err.clone());
                    self.diagnostics.push(DiagnosticEvent::new(LogLevel::Error, Source::Worker, format!("Connect failed: {}", err)));
                }
                Task::none()
            }
            Message::WorkerDisconnected(_) => { 
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Disconnected"));
                self.connection_status = ConnectionStatus::Disconnected; 
                Task::none() 
            }
            Message::WorkerPulled(result) => {
                self.operation_lock.is_pulling = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
                            self.editor_state.filters = peq.filters; self.editor_state.global_gain = peq.global_gain;
                            self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Pull successful"));
                        }
                    }
                } else if let Some(err) = result.error {
                    if err.contains("Not connected") || err.contains("not found") { self.connection_status = ConnectionStatus::Disconnected; }
                    else { self.connection_status = ConnectionStatus::Error(err.clone()); }
                    self.diagnostics.push(DiagnosticEvent::new(LogLevel::Error, Source::Worker, format!("Pull failed: {}", err)));
                }
                Task::none()
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
                            self.editor_state.filters = peq.filters; self.editor_state.global_gain = peq.global_gain;
                            self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Push successful"));
                        }
                    }
                } else if let Some(err) = result.error {
                    if err.contains("Not connected") || err.contains("not found") { self.connection_status = ConnectionStatus::Disconnected; }
                    else { self.connection_status = ConnectionStatus::Error(err.clone()); }
                    self.diagnostics.push(DiagnosticEvent::new(LogLevel::Error, Source::Worker, format!("Push failed: {}", err)));
                }
                Task::none()
            }
            Message::WorkerStatus(status) => {
                if status.connected && self.connection_status != ConnectionStatus::Connected { self.connection_status = ConnectionStatus::Connected; log::info!("Device connected"); self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Device connected (poll)")); }
                else if !status.connected && self.connection_status == ConnectionStatus::Connected { self.connection_status = ConnectionStatus::Disconnected; log::info!("Device disconnected"); self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::Worker, "Device disconnected (poll)")); }
                Task::none()
            }
            Message::Tick(_) => {
                let worker = match &self.worker { Some(w) => w, None => return Task::none() };
                let worker = Arc::clone(worker);
                Task::perform(async move { let rx = worker.status(); rx.recv().unwrap_or(WorkerStatus { connected: false, physically_present: false }) }, Message::WorkerStatus)
            }
            Message::BandEnabledChanged(index, enabled) => { if let Some(band) = self.editor_state.filters.get_mut(index) { band.enabled = enabled; } Task::none() }
            Message::BandGainChanged(index, gain) => { if let Some(band) = self.editor_state.filters.get_mut(index) { band.gain = gain; band.enabled = true; } Task::none() }
            Message::BandFreqChanged(index, freq) => { if let Some(band) = self.editor_state.filters.get_mut(index) { band.freq = freq; } Task::none() }
            Message::BandQChanged(index, q) => { if let Some(band) = self.editor_state.filters.get_mut(index) { band.q = q; } Task::none() }
            Message::GlobalGainChanged(gain) => { self.editor_state.global_gain = gain; Task::none() }
            Message::ExportAutoEQPressed => {
                let peq = PEQData { filters: self.editor_state.filters.clone(), global_gain: self.editor_state.global_gain };
                let output = autoeq::peq_to_autoeq(&peq);
                self.editor_state.autoeq_message = Some("Exported to clipboard".into());
                let _write_task: iced::Task<()> = clipboard::write(output);
                Task::none()
            }
            Message::ExportComplete => { Task::none() }
            Message::ImportFromClipboard => {
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::AutoEQ, "Import from clipboard started"));
                clipboard::read().map(|result| {
                    match result {
                        Some(text) => Message::ImportClipboardReceived(text),
                        None => Message::ImportClipboardFailed("Clipboard empty or not text".into()),
                    }
                })
            }
            Message::ImportClipboardReceived(text) => {
                match autoeq::parse_autoeq_text(&text) {
                    Ok(peq) => {
                        let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
                        self.editor_state.filters = peq.filters;
                        self.editor_state.global_gain = peq.global_gain;
                        self.editor_state.autoeq_message = Some(format!("Imported {} filters from clipboard", enabled_count));
                        self.diagnostics.push(DiagnosticEvent::new(LogLevel::Info, Source::AutoEQ, format!("Import successful: {} filters", enabled_count)));
                    }
                    Err(e) => {
                        self.editor_state.autoeq_message = Some(format!("Error: {}", e));
                        self.diagnostics.push(DiagnosticEvent::new(LogLevel::Error, Source::AutoEQ, format!("Import failed: {}", e)));
                    }
                }
                Task::none()
            }
            Message::ImportClipboardFailed(msg) => {
                self.editor_state.autoeq_message = Some(msg.clone());
                self.diagnostics.push(DiagnosticEvent::new(LogLevel::Error, Source::AutoEQ, msg));
                Task::none()
            }
            Message::CopyDiagnostics => {
                let conn_str = format!("{:?}", self.connection_status);
                let output = crate::diagnostics::format_diagnostics(&self.diagnostics, "0.1.0", &conn_str);
                let _ = clipboard::write::<()>(output);
                self.editor_state.autoeq_message = Some("Diagnostics copied to clipboard".into());
                Task::none()
            }
            Message::ClearDiagnostics => {
                self.diagnostics.clear();
                self.editor_state.autoeq_message = Some("Diagnostics cleared".into());
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.connection_status {
            ConnectionStatus::Disconnected => text("Disconnected"),
            ConnectionStatus::Connecting => text("Connecting..."),
            ConnectionStatus::Connected => text("Connected"),
            ConnectionStatus::Error(e) => text(format!("Error: {}", e)),
        };
        
        let btn_row = row![
            if self.connection_status == ConnectionStatus::Disconnected || matches!(&self.connection_status, ConnectionStatus::Error(_)) { button("Connect").on_press(Message::ConnectPressed) } else { button("Connect") },
            if self.connection_status == ConnectionStatus::Connected { button("Disconnect").on_press(Message::DisconnectPressed) } else { button("Disconnect") },
            if self.connection_status == ConnectionStatus::Connected && !self.operation_lock.is_pulling && !self.operation_lock.is_pushing { button("Pull").on_press(Message::PullPressed) } else { button("Pull") },
            if self.connection_status == ConnectionStatus::Connected && !self.operation_lock.is_pulling && !self.operation_lock.is_pushing { button("Push").on_press(Message::PushPressed) } else { button("Push") },
        ].spacing(10);
        
        let band_list: Vec<Element<Message>> = self.editor_state.filters.iter().enumerate()
            .map(|(i, band)| {
                let enabled_check = checkbox(band.enabled)
                    .label(format!("Band {}", i))
                    .on_toggle(move |v| Message::BandEnabledChanged(i, v));
                let gain_slider = slider(MIN_BAND_GAIN..=MAX_BAND_GAIN, band.gain, move |v| Message::BandGainChanged(i, v));
                let gain_label = text(format!("{:.1} dB", band.gain));
                let freq_label = text(format!("Freq: {} Hz", band.freq));
                let q_label = text(format!("Q: {:.1}", band.q));
                
                column![
                    enabled_check,
                    row![text(format!("Gain:")), gain_slider.width(Length::FillPortion(3)), gain_label.width(Length::FillPortion(1))].spacing(5),
                    row![freq_label, q_label].spacing(10),
                ].spacing(5).into()
            })
            .collect();
        
        let bands = column(band_list).spacing(10);
        
        let global_gain_row = row![
            text("Preamp:"),
            slider(MIN_GLOBAL_GAIN as f64..=MAX_GLOBAL_GAIN as f64, self.editor_state.global_gain as f64, |v| Message::GlobalGainChanged(v as i8)),
            text(format!("{} dB", self.editor_state.global_gain)),
        ].spacing(10);

        let autoeq_section = column![
            text("AutoEQ").size(16),
            row![
                button("Import from Clipboard").on_press(Message::ImportFromClipboard),
                button("Export to Clipboard").on_press(Message::ExportAutoEQPressed),
            ].spacing(10),
            if let Some(ref msg) = self.editor_state.autoeq_message { text(msg).size(14) } else { text("").size(14) },
        ].spacing(10);

        let diag_events: Vec<Element<Message>> = self.diagnostics.events()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(10)
            .map(|e| {
                let level_str = match e.level {
                    LogLevel::Info => "[INFO]",
                    LogLevel::Warn => "[WARN]",
                    LogLevel::Error => "[ERROR]",
                };
                text(format!("{} {} {}", level_str, e.source, e.message)).size(12).into()
            })
            .collect();
        
        let diag_section = column![
            text("Diagnostics").size(16),
            scrollable(column(diag_events).spacing(4)).height(Length::Fixed(120.0)),
            row![
                button("Copy").on_press(Message::CopyDiagnostics),
                button("Clear").on_press(Message::ClearDiagnostics),
            ].spacing(10),
            text(format!("{} events", self.diagnostics.count())).size(12),
        ].spacing(10);
        
        let content = column![
            text("Frost-Tune").size(24),
            status_text,
            btn_row,
            global_gain_row,
            scrollable(bands).height(Length::FillPortion(2)),
            autoeq_section,
            diag_section,
        ].spacing(10).padding(20);
        
        container(content).width(Length::Fill).height(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        use std::time::Duration;
        use std::pin::Pin;
        use iced::time;
        async fn tick() -> Message { Message::Tick(std::time::Instant::now()) }
        time::repeat(|| Pin::from(Box::pin(tick())), Duration::from_secs(2))
    }
}

pub fn run() -> iced::Result {
    iced::application(MainWindow::new, MainWindow::update, MainWindow::view).title(MainWindow::title).subscription(MainWindow::subscription).theme(Theme::Dark).run()
}