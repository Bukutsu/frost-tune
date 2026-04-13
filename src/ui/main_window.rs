use std::sync::Arc;
use std::time::Instant;
use iced::{
    Element, Task, Subscription, Theme,
    widget::{button, column, text, row, container, scrollable, Slider},
    Length,
};
use crate::hardware::worker::{UsbWorker, WorkerStatus};
use crate::models::{ConnectionResult, OperationResult, PEQData, Filter};

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
    Tick(Instant),
    BandEnabledChanged(usize, bool),
    BandGainChanged(usize, f64),
    BandFreqChanged(usize, u16),
    BandQChanged(usize, f64),
    GlobalGainChanged(i8),
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
            },
            operation_lock: OperationLock::default(),
            worker: Some(worker),
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
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.connect();
                    rx.recv().unwrap_or(ConnectionResult { success: false, device: None, error: Some("Worker closed".into()) })
                }, Message::WorkerConnected)
            }
            Message::DisconnectPressed => {
                if self.worker.is_none() { return Task::none(); }
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.disconnect();
                    rx.recv().unwrap_or(OperationResult { success: false, data: None, error: Some("Worker closed".into()) })
                }, Message::WorkerDisconnected)
            }
            Message::PullPressed => {
                if self.worker.is_none() || self.operation_lock.is_pulling || self.operation_lock.is_pushing { return Task::none(); }
                self.operation_lock.is_pulling = true;
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.pull_peq();
                    rx.recv().unwrap_or(OperationResult { success: false, data: None, error: Some("Worker closed".into()) })
                }, Message::WorkerPulled)
            }
            Message::PushPressed => {
                if self.worker.is_none() || self.operation_lock.is_pulling || self.operation_lock.is_pushing { return Task::none(); }
                self.operation_lock.is_pushing = true;
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
                if result.success { self.connection_status = ConnectionStatus::Connected; }
                else { self.connection_status = ConnectionStatus::Error(result.error.unwrap_or_else(|| "Unknown".into())); }
                Task::none()
            }
            Message::WorkerDisconnected(_) => { self.connection_status = ConnectionStatus::Disconnected; Task::none() }
            Message::WorkerPulled(result) => {
                self.operation_lock.is_pulling = false;
                if result.success {
                    if let Some(data) = result.data {
                        if let Ok(peq) = serde_json::from_value::<PEQData>(data) {
                            self.editor_state.filters = peq.filters; self.editor_state.global_gain = peq.global_gain;
                        }
                    }
                } else if let Some(err) = result.error {
                    if err.contains("Not connected") || err.contains("not found") { self.connection_status = ConnectionStatus::Disconnected; }
                    else { self.connection_status = ConnectionStatus::Error(err); }
                }
                Task::none()
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if !result.success { if let Some(err) = result.error { if err.contains("Not connected") || err.contains("not found") { self.connection_status = ConnectionStatus::Disconnected; } else { self.connection_status = ConnectionStatus::Error(err); } } }
                Task::none()
            }
            Message::WorkerStatus(status) => {
                if status.connected && self.connection_status != ConnectionStatus::Connected { self.connection_status = ConnectionStatus::Connected; log::info!("Device connected"); }
                else if !status.connected && self.connection_status == ConnectionStatus::Connected { self.connection_status = ConnectionStatus::Disconnected; log::info!("Device disconnected"); }
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
        }
    }

    fn view(&self) -> Element<Message> {
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
        
        let info = text(format!("Filters: {} | Preamp: {} dB", self.editor_state.filters.len(), self.editor_state.global_gain));
        
        let band_list: Vec<Element<Message>> = self.editor_state.filters.iter().enumerate()
            .map(|(i, band)| {
                text(format!("{}: {}Hz {:.1}dB Q{:.1} {}", i, band.freq, band.gain, band.q, if band.enabled { "[ON]" } else { "[OFF]" })).into()
            })
            .collect();
        
        let bands = column(band_list).spacing(5);
        
        let content = column![
            text("Frost-Tune").size(24),
            status_text,
            btn_row,
            info,
            scrollable(bands).height(Length::Fixed(300.0)),
        ].spacing(10).padding(20);
        
        container(content).width(Length::Fill).height(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> { Subscription::none() }
}

pub fn run() -> iced::Result {
    iced::application(MainWindow::new, MainWindow::update, MainWindow::view).title(MainWindow::title).subscription(MainWindow::subscription).theme(Theme::Dark).run()
}