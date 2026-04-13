pub mod hardware;
pub mod ui;

use log::info;

fn main() {
    env_logger::init();
    info!("Starting Frost-Tune v0.1.0");
    ui::run();
}