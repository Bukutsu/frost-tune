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
    A native, high-performance parametric EQ editor for USB DACs.
    <br />
    Shape your sound with zero-latency precision and push state directly to hardware.
    <br />
    <br />
    <a href="https://github.com/Bukutsu/frost-tune/releases/latest"><strong>Download Latest Release »</strong></a>
    <br />
    <br />
    <a href="#usage">View Usage</a>
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
      <a href="#about-the-project">About The Project</a>
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

Frost-Tune is a native, cross-platform parametric EQ editor that communicates directly with USB DACs over HID. It runs fully offline, stores profiles locally in the standard AutoEQ format, and ensures hardware safety through a transactional verification model — every write is read back and verified, with automatic rollback on mismatch.

Built with an uncompromising **Industrial Utilitarian** design philosophy: blocky geometry, zero decorative motion, and immediate visual feedback that mirrors hardware reality.

### Key Features

* **Direct Hardware Control** — Real-time communication with DACs over USB HID
* **10-Band Parametric EQ** — Frequency, gain, Q-factor, and filter type per band
* **AutoEQ Support** — Import and export profiles in the industry-standard AutoEQ format
* **Transactional Writes** — Every change is verified by reading back hardware state with automatic rollback on mismatch
* **Offline First** — No cloud dependencies or account required
* **Hardware Safety** — Gain and preamp values hard-capped at ±10 dB to protect your equipment

### Supported Devices

| Device | Vendor ID | Product ID | Status |
|--------|-----------|------------|--------|
| **EPZ TP35 Pro** | `0x3302` | `0x43E6` | ✓ Supported |

> More devices can be added by implementing the `DeviceProtocol` trait. See [Contributing](#contributing) for details.

### Built With

* [![Rust][Rust-badge]][Rust-url]
* [![Iced][Iced-badge]][Iced-url]
* [![Tokio][Tokio-badge]][Tokio-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

#### Linux
* **Rust Toolchain**: 1.75 or higher
* **System Libraries**: `libhidapi-dev`, `pkg-config`
* **Polkit**: Required for privileged USB access (unless udev rules are configured)

  ```sh
  # Debian/Ubuntu
  sudo apt install libhidapi-dev pkg-config

  # Arch Linux
  sudo pacman -S hidapi pkgconf

  # Fedora
  sudo dnf install hidapi-devel pkgconfig
  ```

#### Windows
* **Rust Toolchain**: 1.75 or higher
* **Build Tools**: Visual C++ Build Tools (included with Visual Studio)

### Installation

#### From Releases (Recommended)

Download the latest pre-built binary for your platform from the [Releases page](https://github.com/Bukutsu/frost-tune/releases/latest):

| Platform | Package |
|----------|---------|
| **Linux (generic)** | `frost-tune-vX.Y.Z-x86_64-unknown-linux-gnu` |
| **Debian/Ubuntu** | `frost-tune-vX.Y.Z-amd64.deb` |
| **Fedora/RHEL** | `frost-tune-vX.Y.Z-x86_64.rpm` |
| **Arch Linux** | Build from `PKGBUILD` (see below) |
| **Windows** | `frost-tune-vX.Y.Z-x86_64.msi` |

#### From Source

1. Clone the repo
   ```sh
   git clone https://github.com/Bukutsu/frost-tune.git
   cd frost-tune
   ```
2. Build and run
   ```sh
   cargo run --release
   ```

#### Arch Linux (PKGBUILD)

The PKGBUILD lives under `packaging/arch/` to avoid colliding with Cargo's `src/` layout:

```sh
cd packaging/arch
makepkg -si
```

#### Linux USB Permissions

By default, Linux requires root privileges to access USB HID devices. Frost-Tune will prompt for a Polkit password. To avoid this, install a udev rule for your device:

```sh
# Example for EPZ TP35 Pro (VID: 3302, PID: 43e6)
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", MODE="0666"' \
  | sudo tee /etc/udev/rules.d/99-frosttune.rules

# Reload rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

> If you are using a different device, update the `idVendor` and `idProduct` values to match your DAC (find these in the app header or via `lsusb`).

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE -->
## Usage

1. **Connect** your supported USB DAC
2. **Launch** Frost-Tune — it will detect the device automatically
3. **Read** the current EQ state from hardware
4. **Adjust** bands using the frequency, gain, and Q controls
5. **Write** changes back to the device (automatically verified)
6. **Save/Load** profiles in AutoEQ format for portability

| Command | Description |
|---------|-------------|
| `cargo run` | Start in debug mode |
| `cargo run --release` | Start with optimizations |
| `cargo test --all-targets` | Run the full test suite |
| `cargo fmt --all` | Format the codebase |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ARCHITECTURE -->
## Architecture

```
frost-tune/
├── src/
│   ├── hardware/       # HID communication & protocol layer
│   │   ├── worker/     # Background thread for USB I/O
│   │   └── protocol.rs # DeviceProtocol trait abstraction
│   ├── models/         # Domain types (Filters, Devices, IPC)
│   ├── ui/             # Iced GUI (Elm architecture)
│   │   ├── views/      # Pure view components
│   │   └── update/     # Message handlers
│   ├── autoeq.rs       # AutoEQ format parser
│   ├── storage.rs      # Profile persistence
│   └── main.rs         # Entry point
├── tests/              # Integration & protocol tests
├── packaging/          # OS-specific build configs
└── assets/             # Icons, fonts, screenshots
```

### Request Lifecycle

```
UI Interaction → Message Dispatch → MPSC Channel → USB Worker → HID Packets
                                                         ↓
                 State Sync ← Operation Result ← Read-back Verification
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

- [x] 10-band parametric EQ editing
- [x] AutoEQ profile import/export
- [x] Transactional write verification with rollback
- [x] Cross-platform packaging (`.deb`, `.rpm`, `.msi`, PKGBUILD)
- [ ] Additional device support
- [ ] Preset library with community profiles
- [ ] Frequency response graph overlay comparison

See the [open issues](https://github.com/Bukutsu/frost-tune/issues) for a full list of proposed features and known issues.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also open an issue with the tag "enhancement".

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/amazing-feature`)
3. Commit your Changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the Branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Adding a New Device

1. Implement `DeviceProtocol` trait in `src/hardware/protocol.rs`
2. Register the device in `src/models/device.rs` via the `define_devices!` macro
3. Add integration tests in `tests/protocol.rs`

### Before Submitting

```sh
cargo fmt --all && cargo clippy --all-targets && cargo test --all-targets
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Bukutsu — [@Bukutsu](https://github.com/Bukutsu)

Project Link: [https://github.com/Bukutsu/frost-tune](https://github.com/Bukutsu/frost-tune)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Iced](https://iced.rs/) — The native Rust GUI framework
* [hidapi](https://github.com/libusb/hidapi) — Cross-platform HID communication
* [Best-README-Template](https://github.com/othneildrew/Best-README-Template) — README structure and layout

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
