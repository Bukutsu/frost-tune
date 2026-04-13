pub mod autoeq;
pub mod diagnostics;
pub mod error;
pub mod hardware;
pub mod models;
pub mod storage;
pub mod ui;

use log::info;

fn main() {
    env_logger::init();
    info!("Starting Frost-Tune v0.1.0");
    if let Err(e) = ui::run() {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}
