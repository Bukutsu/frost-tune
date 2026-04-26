#[cfg(target_os = "linux")]
fn main() {
    if let Err(e) = frost_tune::hardware::helper_server::run() {
        eprintln!("frost-tune-hid-helper error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("frost-tune-hid-helper is only supported on Linux");
    std::process::exit(1);
}
