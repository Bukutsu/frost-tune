<a id="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<br />
<div align="center">
  <a href="https://github.com/Bukutsu/frost-tune">
    <img src="assets/frost-tune.svg" alt="Frost-Tune" width="80" height="80">
  </a>

## Frost-Tune

Native parametric EQ editor for USB DACs. Offline, transactional, zero-latency.

[Download](https://github.com/Bukutsu/frost-tune/releases/latest) · [Usage](#usage) · [Report Bug](https://github.com/Bukutsu/frost-tune/issues/new?labels=bug) · [Request Feature](https://github.com/Bukutsu/frost-tune/issues/new?labels=enhancement)
</div>

## Table of Contents

- [About The Project](#about-the-project)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Acknowledgments](#acknowledgments)

## About The Project

![Frost-Tune Screenshot][product-screenshot]

Frost-Tune talks to compatible USB DACs over HID, edits 10-band parametric EQ settings, and verifies every write before it sticks.

### Key Features

- Direct USB HID control
- 10-band parametric EQ
- AutoEQ import/export
- Read-back verification with rollback
- Offline, no account, no cloud
- ±10 dB cap on gain and preamp

### Built With

- Rust
- Iced
- Tokio

## Getting Started

### Prerequisites

- Linux: Rust 1.75+, `libhidapi-dev`, `pkg-config`, and polkit or a udev rule
- Windows: Rust 1.75+ and Visual C++ Build Tools

### Installation

- Prebuilt releases: `.deb`, `.rpm`, `.msi`, or raw Linux binary
- From source:

```sh
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune
cargo run --release
```

- Linux helper server:

```sh
cargo run --release -- --hid-helper
```

- Arch Linux:

```sh
cd packaging/arch && makepkg -si
```
### USB Access & Security on Linux

On Linux, raw USB HID access to DACs requires elevated privileges by default.

#### Security & Polkit
To keep your system secure, **Frost-Tune never runs the GUI or main logic as root**:
1. The app starts unprivileged.
2. It uses `pkexec` (Polkit) to request a password *only* to spawn a tiny, non-GUI helper process (`frost-tune --hid-helper`).
3. The helper handles raw USB communication and talks back to the main unprivileged GUI via secure local pipes (JSON IPC).

#### Passwordless Access (udev Rule)
If you prefer not to enter your password or run any code elevated, you can grant your user direct permission to write to the DAC:

1. Create `/etc/udev/rules.d/70-frost-tune.rules` (replace `idVendor`/`idProduct` with your DAC's IDs from `lsusb`):
   ```udev
   # EPZ TP35 Pro (replace with your DAC's VID/PID in lowercase hex if different)
   KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", TAG+="uaccess"
   ```
2. Reload rules:
   ```sh
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```
3. Replug your DAC. Frost-Tune will now run without asking for a password.

## Usage

1. Connect the DAC.
2. Launch Frost-Tune.
3. Read the current hardware state.
4. Adjust bands, frequency, gain, Q, and filter type.
5. Write changes and let the app verify them.
6. Save or load AutoEQ profiles.

Common commands:

| Command | Purpose |
|---|---|
| `cargo run --release` | Launch the app |
| `cargo fmt --all` | Format |
| `cargo clippy --all-targets -- -D warnings` | Lint |
| `cargo build --all-targets --locked` | Build |
| `cargo test --all-targets --locked` | Test |

## Architecture

```text
src/
├── hardware/    HID, protocol, worker, Linux helper
├── models/      Devices, filters, IPC
├── ui/          Iced views and update logic
├── storage.rs   Profiles, diagnostics, settings
└── main.rs      App entry point
```

```text
UI -> Message -> Worker -> HID packets -> Device
                 |-> read-back verify -> state
```

On Linux, `--hid-helper` starts the helper server used for elevated HID access.

## Roadmap

- More devices
- Frequency response overlay
- Community presets

## Contributing

Add a device by implementing `DeviceProtocol`, registering it in `Device`, and adding packet tests.

Before submitting:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --all-targets --locked
cargo test --all-targets --locked
```

## License

MIT. See `LICENSE`.

## Contact

Bukutsu — [@Bukutsu](https://github.com/Bukutsu)

Project: https://github.com/Bukutsu/frost-tune

## Acknowledgments

- [Iced](https://iced.rs/)
- [hidapi](https://github.com/libusb/hidapi)
- [Best-README-Template](https://github.com/othneildrew/Best-README-Template)

[contributors-shield]: https://img.shields.io/github/contributors/Bukutsu/frost-tune.svg?style=for-the-badge
[contributors-url]: https://github.com/Bukutsu/frost-tune/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Bukutsu/frost-tune.svg?style=for-the-badge
[forks-url]: https://github.com/Bukutsu/frost-tune/network/members
[stars-shield]: https://img.shields.io/github/stars/Bukutsu/frost-tune.svg?style=for-the-badge
[stars-url]: https://github.com/Bukutsu/frost-tune/stargazers
[issues-shield]: https://img.shields.io/github/issues/Bukutsu/frost-tune.svg?style=for-the-badge
[issues-url]: https://github.com/Bukutsu/frost-tune/issues
[license-shield]: https://img.shields.io/github/license/Bukutsu/frost-tune.svg?style=for-the-badge
[license-url]: https://github.com/Bukutsu/frost-tune/blob/main/LICENSE
[product-screenshot]: assets/screenshot.png
