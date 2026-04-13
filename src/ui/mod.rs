pub mod main_window;
pub mod graph;
pub mod theme;

pub use main_window::*;

pub fn run() -> iced::Result {
    main_window::run()
}