# Frost-Tune

Native cross-platform parametric EQ editor for USB DACs.

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Iced-0.14-blue?style=flat-square" alt="Iced">
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-lightgrey?style=flat-square&logo=linux&logoColor=white" alt="Platform">
  <img src="https://img.shields.io/badge/HID-hidapi-yellow?style=flat-square" alt="HID Library">
  <img src="https://img.shields.io/badge/Theme-Tokyo%20Night-purple?style=flat-square" alt="Theme">
</p>

## About

Frost-Tune is a native desktop application for configuring parametric equalizer (PEQ) settings on USB DACs. Built with **Rust** and **Iced** GUI framework, it provides a modern, cross-platform experience without the overhead of Electron or WebView. The application communicates directly with DAC hardware over USB HID, providing precise control over audio equalization parameters.

## Features

- **Native Performance**: Built with Rust for blazing-fast speed and small binary size
- **Modern UI**: Iced-based UI with Tokyo Night theme support following Material Design 3 principles
- **Multi-DAC Support**: Extensible architecture supporting multiple DAC protocols (starting with EPZ TP35 Pro)
- **Transactional Updates**: Safe push mechanism with write → read-back → verify pattern and automatic rollback
- **Profile Management**: Import/export EQ configurations in standard formats compatible with AutoEQ and REW
- **Persistent Storage**: Save equalizer configurations directly to DAC flash memory
- **Offline First**: No internet required, runs completely offline
- **Cross-Platform**: Supports Linux and Windows with planned macOS support

## Supported Devices

| DAC | Vendor ID | Product ID | Status |
|-----|-----------|------------|--------|
| EPZ TP35 Pro | 0x3302 | 0x43E6 | ✅ Supported |

## Screenshots

![Frost-Tune Main Interface](assets/screenshot.png)

*Parametric equalizer interface showing 10-band PEQ editor with frequency, Q, and gain controls*

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
│   ├── lib.rs               # Core library exports and module definitions
│   ├── models.rs            # Data structures (Filter, PEQData, DeviceInfo)
│   ├── storage.rs           # Persistent storage for EQ profiles
│   ├── error.rs             # Error handling definitions
│   ├── diagnostics.rs       # Diagnostic and logging utilities
│   ├── autoeq.rs            # AutoEQ profile parsing utilities
│   ├── hardware/             # USB HID communication and DSP logic
│   │   ├── mod.rs           # Hardware module exports
│   │   ├── worker.rs        # Background thread for hidapi operations
│   │   ├── protocol.rs      # Raw USB packet building and parsing
│   │   ├── dsp.rs           # Frequency/Q/Gain to Biquad math conversion
│   │   ├── hid.rs           # HID device read/write operations
│   │   └── packet_builder.rs # Command packet construction utilities
│   └── ui/                   # Iced GUI components
│       ├── mod.rs           # UI module exports
│       ├── main_window.rs   # Main application window and layout
│       ├── state.rs         # Application state management
│       ├── messages.rs      # UI message definitions
│       ├── graph.rs         # Frequency response visualization
│       ├── theme.rs         # Tokyo Night theme implementation
│       └── tokens.rs        # Theme color tokens
├── Cargo.toml
├── AGENTS.md                # Developer instructions and coding standards
└── README.md
```

### Commands

```bash
# Build debug version
cargo build

# Run in development mode
cargo run

# Check for errors
cargo check

# Run tests
cargo test

# Build release version
cargo build --release

# Build with optimizations
cargo build --release
```

### Safety Features

Frost-Tune implements multiple layers of safety for audio hardware control:

1. **Transactional Push**: All EQ updates follow `Write → Read Back → Verify` pattern
2. **Automatic Rollback**: If verification fails, device state is rolled back to previous
3. **Safety Limits**: Band gain capped at +10dB, global preamp at +10dB
4. **Timing Gaps**: Proper delays between packets prevent flash corruption
5. **Unified Threading Model**: All HID calls happen in background thread, never from UI widgets

### Tech Stack

- **Language**: Rust 1.75+ (Edition 2021)
- **GUI Framework**: [Iced](https://iced.rs/) 0.14
- **HID Library**: [hidapi](https://github.com/libusb/hidapi)
- **Logging**: [env_logger](https://crates.io/crates/env_logger) with log crate
- **Theme**: Built-in Tokyo Night (with full theme support)
- **Build System**: Cargo

### Cross-Platform Notes

- **Linux**: Requires `libhidapi-dev` (`sudo apt install libhidapi-dev`)
- **Windows**: Requires WebView2 (usually pre-installed on Win10/11)
- **macOS**: Planned support (currently untested)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Iced](https://iced.rs/) - Cross-platform GUI library for Rust
- [hidapi](https://github.com/libusb/hidapi) - Cross-platform HID library
- Original [tp35pro-eq](https://github.com/Bukutsu/tp35pro-eq) project for hardware protocol reference
- Material Design 3 guidelines for UI/UX principles

## Contributing

Contributions are welcome! Please read [AGENTS.md](AGENTS.md) for developer instructions, coding standards, and project architecture details before contributing.