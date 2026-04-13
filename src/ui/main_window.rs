use std::sync::Arc;
use iced::{
    Element, Task, Subscription,
    widget::{button, column, text, row, container},
    Theme, Length,
};
use crate::hardware::worker::UsbWorker;
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
    AutoConnect,
}

#[derive(Debug, Clone)]
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
        let window = MainWindow {
            connection_status: ConnectionStatus::Disconnected,
            editor_state: EditorState::default(),
            operation_lock: OperationLock::default(),
            worker: Some(worker),
        };
        (window, Task::none())
    }

    fn title(&self) -> String {
        "Frost-Tune".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConnectPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                self.connection_status = ConnectionStatus::Connecting;
                self.operation_lock.is_connecting = true;
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.connect();
                    rx.recv().unwrap_or(ConnectionResult {
                        success: false,
                        device: None,
                        error: Some("Worker channel closed".into()),
                    })
                }, Message::WorkerConnected)
            }
            Message::DisconnectPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.disconnect();
                    rx.recv().unwrap_or(OperationResult {
                        success: false,
                        data: None,
                        error: Some("Worker channel closed".into()),
                    })
                }, Message::WorkerDisconnected)
            }
            Message::PullPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                if self.operation_lock.is_pulling || self.operation_lock.is_pushing {
                    return Task::none();
                }
                self.operation_lock.is_pulling = true;
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.pull_peq();
                    rx.recv().unwrap_or(OperationResult {
                        success: false,
                        data: None,
                        error: Some("Worker channel closed".into()),
                    })
                }, Message::WorkerPulled)
            }
            Message::PushPressed => {
                if self.worker.is_none() {
                    return Task::none();
                }
                if self.operation_lock.is_pulling || self.operation_lock.is_pushing {
                    return Task::none();
                }
                self.operation_lock.is_pushing = true;
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                let filters = self.editor_state.filters.clone();
                let global_gain = self.editor_state.global_gain;
                Task::perform(async move {
                    use crate::models::PushPayload;
                    let payload = PushPayload { filters, global_gain: Some(global_gain) };
                    let rx = worker.push_peq(payload);
                    rx.recv().unwrap_or(OperationResult {
                        success: false,
                        data: None,
                        error: Some("Worker channel closed".into()),
                    })
                }, Message::WorkerPushed)
            }
            Message::WorkerConnected(result) => {
                self.operation_lock.is_connecting = false;
                if result.success {
                    self.connection_status = ConnectionStatus::Connected;
                } else {
                    self.connection_status = ConnectionStatus::Error(
                        result.error.unwrap_or_else(|| "Unknown error".into())
                    );
                }
                Task::none()
            }
            Message::WorkerDisconnected(_result) => {
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
                        }
                    }
                } else if let Some(err) = result.error {
                    self.connection_status = ConnectionStatus::Error(err);
                }
                Task::none()
            }
            Message::WorkerPushed(result) => {
                self.operation_lock.is_pushing = false;
                if !result.success {
                    if let Some(err) = result.error {
                        self.connection_status = ConnectionStatus::Error(err);
                    }
                }
                Task::none()
            }
            Message::AutoConnect => {
                if self.worker.is_none() {
                    return Task::none();
                }
                self.connection_status = ConnectionStatus::Connecting;
                self.operation_lock.is_connecting = true;
                let worker = Arc::clone(self.worker.as_ref().unwrap());
                Task::perform(async move {
                    let rx = worker.connect();
                    rx.recv().unwrap_or(ConnectionResult {
                        success: false,
                        device: None,
                        error: Some("Worker channel closed".into()),
                    })
                }, Message::WorkerConnected)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let status_text = match &self.connection_status {
            ConnectionStatus::Disconnected => text("Status: Disconnected"),
            ConnectionStatus::Connecting => text("Status: Connecting..."),
            ConnectionStatus::Connected => text("Status: Connected"),
            ConnectionStatus::Error(e) => text(format!("Error: {}", e)),
        };

        let connect_btn = button("Connect")
            .on_press(Message::ConnectPressed);
        let disconnect_btn = button("Disconnect")
            .on_press(Message::DisconnectPressed);
        let pull_btn = button("Pull")
            .on_press(Message::PullPressed);
        let push_btn = button("Push")
            .on_press(Message::PushPressed);

        let btn_row = row![
            connect_btn,
            disconnect_btn,
            pull_btn,
            push_btn,
        ]
        .spacing(10);

        let filter_count = self.editor_state.filters.len();
        let gain = self.editor_state.global_gain;
        let info_text = text(format!("Filters: {} | Global Gain: {} dB", filter_count, gain));

        let content = column![
            text("Frost-Tune").size(24),
            status_text,
            btn_row,
            info_text,
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

pub fn run() -> iced::Result {
    iced::application(
        MainWindow::new,
        MainWindow::update,
        MainWindow::view,
    )
    .title(MainWindow::title)
    .subscription(MainWindow::subscription)
    .theme(Theme::Dark)
    .run()
}