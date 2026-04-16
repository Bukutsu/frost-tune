use frost_tune::ui;
use log::info;

fn main() {
    env_logger::init();
    info!("Starting Frost-Tune v{}", env!("CARGO_PKG_VERSION"));
    if let Err(e) = ui::run() {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}
