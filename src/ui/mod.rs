pub mod main_window;

pub use main_window::*;

use iced::{Application, Settings};

pub fn run() {
    let _ = App::run(Settings::default());
}

struct App;

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (App, iced::Command::none())
    }

    fn title(&self) -> String {
        "Frost-Tune".to_string()
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        use iced::widget::{text, column};
        
        column![text("Frost-Tune v0.1.0")].into()
    }
}

#[derive(Debug, Clone)]
enum Message {}