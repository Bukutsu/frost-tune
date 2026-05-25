// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

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

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "probe" {
        let hex = args.iter().any(|a| a == "--hex");
        let positional: Vec<&str> = args
            .iter()
            .skip(2)
            .filter(|a| !a.starts_with('-'))
            .map(|s| s.as_str())
            .collect();

        if positional.len() < 2 {
            eprintln!("Usage: frost-tune probe <vid> <pid> [--hex]");
            eprintln!("  vid, pid: USB Vendor/Product ID in hex (e.g. 3302 43E6)");
            eprintln!("  --hex:    dump raw hex packet info");
            std::process::exit(1);
        }

        let vid = match u16::from_str_radix(positional[0], 16) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid vendor ID '{}': {}", positional[0], e);
                std::process::exit(1);
            }
        };
        let pid = match u16::from_str_radix(positional[1], 16) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid product ID '{}': {}", positional[1], e);
                std::process::exit(1);
            }
        };

        match frost_tune::probe::run(frost_tune::probe::ProbeOptions { vid, pid, hex }) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Probe failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    env_logger::init();
    info!("Starting Frost-Tune v{}", env!("CARGO_PKG_VERSION"));
    let recent = frost_tune::storage::load_recent_diagnostics(200).unwrap_or_default();
    if let Err(e) = ui::run_with_diagnostics(recent) {
        eprintln!("Error running Frost-Tune: {}", e);
        std::process::exit(1);
    }
}
