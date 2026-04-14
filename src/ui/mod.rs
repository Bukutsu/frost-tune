pub mod graph;
pub mod main_window;
pub mod messages;
pub mod state;
pub mod theme;

pub fn run() -> iced::Result {
    main_window::run()
}
