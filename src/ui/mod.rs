pub mod graph;
pub mod main_window;
pub mod messages;
pub mod state;
pub mod theme;
pub mod tokens;
pub mod update;
pub mod views;

pub use main_window::*;
pub use messages::*;
pub use state::*;

pub fn run() -> iced::Result {
    main_window::run()
}

pub fn run_with_diagnostics(recent_logs: Vec<String>) -> iced::Result {
    main_window::run_with_diagnostics(recent_logs)
}
