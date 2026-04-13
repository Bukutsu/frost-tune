pub mod main_window;

pub use main_window::*;

pub fn run() -> iced::Result {
    main_window::run()
}