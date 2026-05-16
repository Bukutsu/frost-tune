use frost_tune::ui;
use log::info;

fn main() {
    #[cfg(target_os = "linux")]
    {
        if std::env::args().any(|arg| arg == "--hid-helper") {
            if let Err(e) = frost_tune::hardware::helper_server::run() {
                eprintln!("frost-tune --hid-helper error: {}", e);
                std::process::exit(1);
            }
            return;
        }
    }

    env_logger::init();
    info!("Starting Frost-Tune v{}", env!("CARGO_PKG_VERSION"));
    let recent = frost_tune::storage::load_recent_diagnostics(200).unwrap_or_default();
    if let Err(e) = ui::run_with_diagnostics(recent) {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}
