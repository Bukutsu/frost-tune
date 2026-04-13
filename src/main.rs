pub mod error;
pub mod hardware;
pub mod models;
pub mod ui;
pub mod autoeq;

use log::info;

fn main() {
    env_logger::init();
    info!("Starting Frost-Tune v0.1.0");
    if let Err(e) = ui::run() {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}