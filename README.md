<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/Bukutsu/frost-tune">
    <img src="assets/frost-tune.svg" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">Frost-Tune</h3>

  <p align="center">
    Native parametric EQ editor for USB DACs. Offline, transactional, zero-latency.
    <br />
    <br />
    <a href="https://github.com/Bukutsu/frost-tune/releases/latest"><strong>Download »</strong></a>
    <br />
    <br />
    <a href="#usage">Usage</a>
    ·
    <a href="https://github.com/Bukutsu/frost-tune/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    ·
    <a href="https://github.com/Bukutsu/frost-tune/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About</a>
      <ul>
        <li><a href="#key-features">Key Features</a></li>
        <li><a href="#supported-devices">Supported Devices</a></li>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#architecture">Architecture</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

![Frost-Tune Screenshot][product-screenshot]

Talks HID directly to USB DACs. Runs offline, stores AutoEQ-compatible profiles. Every write is read back and verified; mismatches roll back automatically.

Industrial Utilitarian UI — blocky geometry, no decorative motion.

### Key Features

* Direct USB HID — no driver shim
* 10-band parametric EQ (frequency, gain, Q, type per band)
* AutoEQ import/export
* Transactional writes with automatic rollback
* Offline, no account, no cloud
* ±10 dB hard cap on gain and preamp

### Supported Devices

| Device | VID | PID | Status |
|---|---|---|---|
| EPZ TP35 Pro | `0x3302` | `0x43E6` | ✓ |

> Add devices by implementing the `DeviceProtocol` trait — see [Contributing](#contributing).

### Built With

* [![Rust][Rust-badge]][Rust-url]
* [![Iced][Iced-badge]][Iced-url]
* [![Tokio][Tokio-badge]][Tokio-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

| Platform | Needs |
|---|---|
| Linux | Rust ≥ 1.75, `libhidapi-dev`, `pkg-config`, polkit (or udev rule) |
| Windows | Rust ≥ 1.75, Visual C++ Build Tools |

```sh
# Debian/Ubuntu
sudo apt install libhidapi-dev pkg-config
# Arch
sudo pacman -S hidapi pkgconf
# Fedora
sudo dnf install hidapi-devel pkgconfig
```

### Installation

**Pre-built:** grab a `.deb`, `.rpm`, `.msi`, or raw Linux binary from [Releases](https://github.com/Bukutsu/frost-tune/releases/latest).

**From source:**

```sh
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune
cargo run --release
```

**Arch (PKGBUILD lives under `packaging/arch/` to avoid colliding with Cargo's `src/`):**

```sh
cd packaging/arch && makepkg -si
```

**Skip the polkit prompt** with a udev rule:

```sh
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", MODE="0666"' \
  | sudo tee /etc/udev/rules.d/99-frosttune.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

> Adjust `idVendor` / `idProduct` for other devices — find them in the app header or via `lsusb`.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE -->
## Usage

1. Connect the DAC
2. Launch — device is detected automatically
3. **Read** state from hardware
4. **Adjust** bands (frequency / gain / Q)
5. **Write** — verified automatically
6. **Save/Load** profiles in AutoEQ format

| Command | What it does |
|---|---|
| `cargo run --release` | Launch |
| `cargo test --all-targets` | Tests |
| `cargo fmt --all` | Format |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ARCHITECTURE -->
## Architecture

```
src/
├── hardware/   # HID + protocol layer; worker/ owns USB I/O off the UI thread
├── models/    # Filters, Devices, IPC
├── ui/        # Iced (Elm-style); views/ pure, update/ mutates
├── autoeq.rs  # AutoEQ format parser
├── storage.rs # Profile + settings persistence
└── main.rs
```

```
UI → Message → MPSC → Worker → HID Packets
                         ↓
       State ← Result ← Read-back Verify
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

- [x] 10-band parametric EQ editing
- [x] AutoEQ profile import/export
- [x] Transactional writes with rollback
- [x] Cross-platform packaging (`.deb`, `.rpm`, `.msi`, PKGBUILD)
- [ ] Additional devices
- [ ] Community preset library
- [ ] Frequency response overlay comparison

See [open issues](https://github.com/Bukutsu/frost-tune/issues) for the full list.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

PRs welcome. Fork → branch → commit → PR.

### Adding a Device

1. Implement `DeviceProtocol` in `src/hardware/protocol.rs`
2. Register via the `define_devices!` macro in `src/models/device.rs`
3. Add packet tests in `tests/protocol.rs`

### Before Submitting

```sh
cargo fmt --all && \
cargo clippy --all-targets -- -D warnings && \
cargo test --all-targets --locked
```

Mirrors CI. See `AGENTS.md` for the full pre-push checklist.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->
## License

MIT. See `LICENSE`.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Bukutsu — [@Bukutsu](https://github.com/Bukutsu)
Project: [github.com/Bukutsu/frost-tune](https://github.com/Bukutsu/frost-tune)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Iced](https://iced.rs/) — Native Rust GUI
* [hidapi](https://github.com/libusb/hidapi) — Cross-platform HID

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
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
[Rust-badge]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Iced-badge]: https://img.shields.io/badge/Iced_0.14-2B2D42?style=for-the-badge
[Iced-url]: https://iced.rs/
[Tokio-badge]: https://img.shields.io/badge/Tokio-1E1E2E?style=for-the-badge
[Tokio-url]: https://tokio.rs/
