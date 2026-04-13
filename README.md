# Frost-Tune

Native cross-platform parametric EQ editor for USB DACs.

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Iced-0.14-blue?style=flat-square" alt="Iced">
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square">
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-lightgrey?style=flat-square&logo=linux&logoColor=white">
</p>

## About

Frost-Tune is a native desktop application for configuring parametric equalizer (PEQ) settings on USB DACs. Built with **Rust** and **Iced** GUI framework, it provides a modern, cross-platform experience without the overhead of Electron or WebView.

## Features

- **Native Performance**: Built with Rust for blazing-fast speed and small binary size
- **Modern UI**: Iced-based UI with Tokyo Night theme support
- **Multi-DAC Support**: Architecture supports multiple DAC protocols (starting with EPZ TP35 Pro)
- **Transactional Updates**: Safe push mechanism with read-back verification and rollback
- **Offline First**: No internet required, runs completely offline
- **Cross-Platform**: Supports Linux and Windows

## Supported Devices

| DAC | Vendor ID | Product ID | Status |
|-----|-----------|------------|--------|
| EPZ TP35 Pro | 0x3302 | 0x43E6 | ✅ Supported |

## Screenshots

```
┌─────────────────────────────────────────────────────────────┐
│  Frost-Tune                                         ✕ ─ □   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    TP35 Pro                        │   │
│   │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐      │   │
│   │  │ 31Hz │ │ 63Hz │ │125Hz │ │250Hz │ │500Hz │ ...  │   │
│   │  │ ──── │ │ ──── │ │ ──── │ │ ──── │ │ ──── │      │   │
│   │  │  0dB │ │  0dB │ │  0dB │ │  0dB │ │  0dB │      │   │
│   │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘      │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   [ Connect ]  [ Pull from Device ]  [ Push to Device ]   │
│                                                             │
│   Status: Disconnected                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune

# Build
cargo build --release

# Run
cargo run --release
```

### Dependencies

**Linux:**
```bash
sudo apt install libhidapi-dev
```

**Windows:**
- WebView2 (pre-installed on Windows 10/11)
- Microsoft Visual C++ Build Tools

## Development

### Project Structure

```
frost-tune/
├── src/
│   ├── main.rs              # Application entry point
│   ├── models.rs            # Data structures (Filter, PEQData, etc.)
│   ├── hardware/             # USB HID communication layer
│   │   ├── dsp.rs           # Biquad filter math
│   │   ├── protocol.rs      # USB packet definitions
│   │   ├── hid.rs           # HID read/write operations
│   │   ├── packet_builder.rs # Command packet construction
│   │   └── worker.rs        # Background USB worker thread
│   └── ui/                   # Iced GUI components
│       ├── mod.rs
│       └── main_window.rs
├── Cargo.toml
├── AGENTS.md                # Developer instructions
└── README.md
```

### Commands

```bash
# Build
cargo build

# Run in development mode
cargo run

# Check for errors
cargo check

# Run tests
cargo test

# Build release
cargo build --release
```

## Safety Features

Frost-Tune implements multiple layers of safety for audio hardware control:

1. **Transactional Push**: All EQ updates follow `Write → Read Back → Verify` pattern
2. **Automatic Rollback**: If verification fails, device state is rolled back to previous
3. **Safety Limits**: Band gain capped at +10dB, global preamp at +10dB
4. **Timing Gaps**: Proper delays between packets prevent flash corruption

## Tech Stack

- **Language**: Rust 1.75+
- **GUI Framework**: [Iced](https://iced.rs/) 0.14
- **HID Library**: [hidapi](https://github.com/libusb/hidapi)
- **Theme**: Built-in Tokyo Night (with full theme support)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Iced](https://iced.rs/) - Cross-platform GUI library for Rust
- [hidapi](https://github.com/libusb/hidapi) - Cross-platform HID library
- Original [tp35pro-eq](https://github.com/Bukutsu/tp35pro-eq) project for hardware protocol reference