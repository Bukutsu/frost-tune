# Frost-Tune — Agent Guidelines

## Project Overview

Frost-Tune is a native parametric EQ editor for USB DACs, built with Rust and the Iced GUI framework. It communicates with DACs over USB HID to adjust 10-band parametric EQ directly on hardware.

**Tech stack:** Rust 2021, Iced 0.14 (GUI), hidapi (HID I/O), tokio (async), serde/serde_json (serialization)

**Target platforms:** Linux (primary), Windows

## Architecture

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library root
├── autoeq.rs            # AutoEQ profile format
├── diagnostics.rs       # Device diagnostics
├── error.rs             # Error types
├── storage.rs           # Profile persistence
├── hardware/            # HID/protocol layer
│   ├── mod.rs
│   ├── dsp.rs           # Biquad filter computation
│   ├── elevated_transport.rs  # Linux privilege escalation
│   ├── helper_ipc.rs    # Helper process IPC
│   ├── helper_server.rs # Elevated helper server
│   ├── hid.rs           # HID transport
│   ├── operations.rs    # High-level hardware ops
│   ├── packet_builder.rs
│   ├── pipeline.rs      # Read/write pipeline
│   ├── protocol.rs      # DeviceProtocol trait + TP35ProProtocol
│   └── worker/          # Background worker thread
├── models/              # Domain types
│   ├── mod.rs
│   ├── constants.rs     # EQ limits, band count, etc.
│   ├── device.rs        # Device definitions + registration
│   ├── filter.rs        # Filter model (freq, gain, Q, type)
│   └── ipc.rs           # IPC message types
└── ui/                  # Iced GUI
    ├── mod.rs
    ├── graph.rs         # Frequency response curve rendering
    ├── main_window.rs   # Window layout
    ├── messages.rs      # Message enum
    ├── state.rs         # App state
    ├── theme.rs         # Styling
    ├── tokens.rs        # Design tokens
    ├── update/          # Message handlers
    └── views/           # UI view components
```

## Key Commands

```bash
cargo fmt                          # Format code (run before committing)
cargo check --all-targets          # Fast compile check
cargo clippy --all-targets         # Lint
cargo test --all-targets           # Run tests
cargo run --release                # Run the app
```

## Code Conventions

- **Edition:** Rust 2021
- **No comments** unless explicitly requested
- **Error handling:** Use `thiserror` for error types, defined in `error.rs`
- **Async:** tokio runtime for background HID I/O; UI runs on main thread
- **HID I/O:** Always isolated on background threads, never on UI thread
- **Transactional writes:** Every EQ write follows push → read-back → verify → rollback-on-mismatch
- **Safety caps:** Band gain and global preamp capped at ±10 dB

## Testing

- Integration tests in `tests/`
- Run with `cargo test --all-targets`
- Protocol tests validate packet construction/parsing for TP35Pro
- Token consistency tests ensure UI design tokens match

## Adding New Devices

1. Implement `DeviceProtocol` trait in `src/hardware/protocol.rs`
2. Register the device in `src/models/device.rs`
3. Follow the contributor guide comments at the bottom of `device.rs`

## Important Patterns

- **DeviceProtocol trait:** Defines the HID packet protocol per device (build read/write packets, parse responses)
- **Iced Elm architecture:** State + Messages + Update + View pattern in `ui/`
- **AutoEQ format:** Profiles stored as plain text, compatible with AutoEQ ecosystem
- **Linux elevation:** Uses `pkexec` with the binary itself as a temporary helper; no system-wide install needed
