use iced::{Element, Column, Text, button, Button, Row, Container};

#[derive(Debug, Clone)]
pub enum Message {
    ConnectPressed,
    DisconnectPressed,
    PullPressed,
    PushPressed,
}

pub struct MainWindow {
    connected: bool,
    btn_connect: button::State,
    btn_disconnect: button::State,
    btn_pull: button::State,
    btn_push: button::State,
}

impl MainWindow {
    pub fn new() -> Self {
        MainWindow {
            connected: false,
            btn_connect: button::State::new(),
            btn_disconnect: button::State::new(),
            btn_pull: button::State::new(),
            btn_push: button::State::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ConnectPressed => {
                self.connected = true;
            }
            Message::DisconnectPressed => {
                self.connected = false;
            }
            Message::PullPressed => {}
            Message::PushPressed => {}
        }
    }

    pub fn view(&self) -> Element<Message> {
        let controls = Row::new()
            .push(Button::new(&mut self.btn_connect, Text::new("Connect"))
                .on_press(Message::ConnectPressed))
            .push(Button::new(&mut self.btn_disconnect, Text::new("Disconnect"))
                .on_press(Message::DisconnectPressed))
            .push(Button::new(&mut self.btn_pull, Text::new("Pull from Device"))
                .on_press(Message::PullPressed))
            .push(Button::new(&mut self.btn_push, Text::new("Push to Device"))
                .on_press(Message::PushPressed));

        let status = if self.connected {
            Text::new("Status: Connected")
        } else {
            Text::new("Status: Disconnected")
        };

        Container::new(
            Column::new()
                .push(Text::new("Frost-Tune"))
                .push(Text::new("v0.1.0"))
                .spacing(20)
                .push(controls)
                .push(status)
        )
        .padding(20)
        .into()
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}