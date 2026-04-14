use frost_tune::ui;
use log::info;

fn main() {
    env_logger::init();
    info!("Starting Frost-Tune v0.1.5");
    if let Err(e) = ui::run() {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}
