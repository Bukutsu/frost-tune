# Frost-Tune

A native parametric EQ editor for USB DACs. Plug in your device, shape your sound, push it to hardware.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust)
![Iced](https://img.shields.io/badge/Iced-0.14-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-lightgrey?style=flat-square)

![Frost-Tune Main Interface](assets/screenshot.png)

## What it does

Frost-Tune talks to your DAC over USB HID and lets you tweak a 10-band parametric EQ directly on the device — frequency, gain, Q, filter type, the works. It runs fully offline, stores profiles locally in AutoEQ-compatible format, and verifies every write by reading back the hardware state.

### Supported devices

| Device | VID / PID | Status |
|--------|-----------|--------|
| EPZ TP35 Pro | `0x3302` / `0x43E6` | ✓ Supported |

More devices can be added by implementing the `DeviceProtocol` trait. See `src/hardware/protocol.rs`.

## Hardware safety

Every write to the device follows a transactional model:

1. Push EQ payload → read back state → verify match → auto-rollback on mismatch

Band gain and global preamp are both hard-capped at ±10 dB. All HID I/O runs on a dedicated background thread, isolated from the UI.

## Getting started

### Prerequisites

- [Rust toolchain](https://rustup.rs) (1.75+)
- Linux: `libhidapi-dev`
- Windows: Visual C++ build tools

### Build and run

```bash
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune
cargo run --release
```

## Linux permissions

Frost-Tune is self-contained and handles elevated HID access automatically. If direct hardware access is denied, it will prompt for elevation via `pkexec` (using the binary itself as a temporary helper). No system-wide installation or complex configuration is required.

<details>
<summary>Alternative: udev rules (to avoid password prompts)</summary>

If you prefer not to enter a password every time you connect a device:

```bash
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \
ATTRS{idProduct}=="43e6", MODE="0666"' \
| sudo tee /etc/udev/rules.d/99-frosttune.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

</details>

## Usage

1. Connect a supported DAC
2. Launch Frost-Tune
3. Adjust PEQ bands — frequency, gain, Q, filter type
4. Preview the response curve
5. Push to device (verification is automatic)
6. Save or export profiles as needed

Profiles are stored as plain text in AutoEQ format and can be imported/exported freely.

## Contributing

Contributions are welcome. Please run `cargo fmt && cargo check && cargo test` before opening a PR.

## Roadmap

- Support for additional USB DACs
- Better profile management and diagnostics
- Broader cross-platform packaging

## Acknowledgments

- [Iced](https://iced.rs/) — native Rust GUI
- [hidapi](https://github.com/libusb/hidapi) — cross-platform HID
- [tp35pro-eq](https://github.com/Bukutsu/tp35pro-eq) — protocol reference
