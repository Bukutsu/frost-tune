<a id="readme-top"></a>

<div align="center">
  <a href="https://github.com/Bukutsu/frost-tune">
    <img src="assets/frost-tune.svg" alt="Frost-Tune" width="80" height="80">
  </a>

## Frost-Tune

Native parametric EQ editor for USB DACs. Offline, transactional, zero-latency.

[![Contributors][contributors-shield]][contributors-url] &nbsp;
[![Forks][forks-shield]][forks-url] &nbsp;
[![Stargazers][stars-shield]][stars-url] &nbsp;
[![Issues][issues-shield]][issues-url] &nbsp;
[![MIT License][license-shield]][license-url]

<br />

[Download](https://github.com/Bukutsu/frost-tune/releases/latest) · [Usage](#usage) · [Report Bug](https://github.com/Bukutsu/frost-tune/issues/new?labels=bug) · [Request Feature](https://github.com/Bukutsu/frost-tune/issues/new?labels=enhancement)
</div>

## Table of Contents

- [About](#about)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## About

![Screenshot][product-screenshot]

Frost-Tune talks to compatible USB DACs over HID, edits 10-band parametric EQ, and verifies every write before it sticks.

**Features**

- Direct USB HID control
- 10-band parametric EQ
- AutoEQ import / export
- Read-back verification with rollback
- Offline — no account, no cloud

### Supported Devices

| Manufacturer | Model | Status | Family / Protocol |
| :--- | :--- | :--- | :--- |
| **EPZ** | TP35 Pro | Tested (Verified) | Walkplay Family |
| **Moondrop** | Dawn Pro | Untested | Walkplay Family |
| **Truthear** | KEYX | Untested | Walkplay Family |

> [!NOTE]
> More devices can be added via the `DeviceProtocol` trait. See `CONTRIBUTING_DEVICES.md` for a comprehensive step-by-step guide.

**Built with:** Rust, Iced, Tokio

## Getting Started

**Prerequisites**

- Linux: Rust 1.75+, `libhidapi-dev`, `pkg-config`
- Windows: Rust 1.75+, Visual C++ Build Tools

**Install**

Download a prebuilt release (`.deb`, `.rpm`, `.msi`, or raw binary), or build from source:

```sh
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune
cargo run --release
```

Arch Linux:

```sh
cd packaging/arch && makepkg -si
```

**Linux USB access**

Frost-Tune never runs the GUI as root. It uses `pkexec` (Polkit) to spawn a tiny helper for raw HID access, then talks back to the main app via JSON IPC.

For passwordless access, add a udev rule:

```udev
# /etc/udev/rules.d/70-frost-tune.rules
# Replace idVendor/idProduct with your DAC's IDs from lsusb
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", TAG+="uaccess"
```

Reload and replug:

```sh
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Usage

1. Plug in the DAC.
2. Launch Frost-Tune.
3. Pull the current hardware state.
4. Edit bands, gain, Q, and filter type.
5. Push changes — the app verifies them automatically.
6. Save or load AutoEQ profiles.

## Contributing

Add new devices by implementing `DeviceProtocol`. See `CONTRIBUTING_DEVICES.md` for the full guide.

Before submitting:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --all-targets --locked
cargo test --all-targets --locked
```

## License

MIT. See `LICENSE`.

## Acknowledgments

- [Iced](https://iced.rs/)
- [hidapi](https://github.com/libusb/hidapi)
- [Best-README-Template](https://github.com/othneildrew/Best-README-Template)
- [devicePEQ](https://github.com/jeromeof/devicePEQ) for reverse-engineered DAC protocols

[contributors-shield]: https://img.shields.io/badge/contributors-2-blue?style=flat&logo=github
[contributors-url]: https://github.com/bukutsu/frost-tune/graphs/contributors
[forks-shield]: https://img.shields.io/badge/forks-0-blue?style=flat&logo=github
[forks-url]: https://github.com/bukutsu/frost-tune/network/members
[stars-shield]: https://img.shields.io/badge/stars-2-brightgreen?style=flat&logo=github
[stars-url]: https://github.com/bukutsu/frost-tune/stargazers
[issues-shield]: https://img.shields.io/badge/issues-0%20open-important?style=flat&logo=github
[issues-url]: https://github.com/bukutsu/frost-tune/issues
[license-shield]: https://img.shields.io/badge/license-MIT-brightgreen?style=flat&logo=github
[license-url]: https://github.com/bukutsu/frost-tune/blob/main/LICENSE
[product-screenshot]: assets/screenshot.png
