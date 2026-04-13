#[derive(Debug, Clone)]
pub enum Message {
    ConnectPressed,
    DisconnectPressed,
    PullPressed,
    PushPressed,
}

pub struct MainWindow;

impl MainWindow {
    pub fn new() -> Self { MainWindow }
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> String {
        "Frost-Tune v0.1.0 - CLI Mode".to_string()
    }
}

impl Default for MainWindow {
    fn default() -> Self { Self::new() }
}