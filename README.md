# Frost-Tune

A native parametric EQ editor for USB DACs. Plug in your device, shape your sound, and push it directly to hardware.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust)
![Iced](https://img.shields.io/badge/Iced-0.14-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-lightgrey?style=flat-square)

![Frost-Tune Main Interface](assets/screenshot.png)

Frost-Tune provides a streamlined, native interface for configuring the internal Parametric EQ (PEQ) of supported USB DACs. It runs fully offline, stores profiles locally in the standard AutoEQ format, and ensures hardware safety through a transactional verification model.

## Key Features

- **Direct Hardware Control**: Real-time communication with DACs over USB HID.
- **10-Band Parametric EQ**: Tweak frequency, gain, Q-factor, and filter type for every band.
- **AutoEQ Support**: Seamlessly import and export profiles in the industry-standard AutoEQ format.
- **Transactional Writes**: Every change is verified by reading back hardware state with automatic rollback on mismatch.
- **Offline First**: No cloud dependencies or account required.
- **Hardware Safety**: Gain and preamp values are hard-capped at ±10 dB to protect your equipment.

## Supported Devices

| Device | Vendor ID | Product ID | Status |
|--------|-----------|------------|--------|
| **EPZ TP35 Pro** | `0x3302` | `0x43E6` | ✓ Supported |

*More devices can be added by implementing the `DeviceProtocol` trait. See the [Architecture](#architecture) section for details.*

---

## Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2021 Edition)
- **GUI Framework**: [Iced](https://iced.rs/) (0.14)
- **USB Communication**: [hidapi](https://github.com/libusb/hidapi)
- **Concurrency**: [Tokio](https://tokio.rs/)
- **Serialization**: [Serde](https://serde.rs/)
- **Styling**: Tokyo Night inspired theme with custom Iced widgets

---

## Prerequisites

### Linux
- **Rust Toolchain**: 1.75 or higher
- **System Libraries**: `libhidapi-dev`, `pkg-config`
- **Polkit**: Required for privileged USB access (unless udev rules are used)

### Windows
- **Rust Toolchain**: 1.75 or higher
- **Build Tools**: Visual C++ Build Tools (included with Visual Studio)

---

## Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/Bukutsu/frost-tune.git
cd frost-tune
```

### 2. Build and Run
```bash
cargo run --release
```

### 3. (Linux Only) Setup USB Permissions
By default, Linux requires root privileges to access USB HID devices. Frost-Tune will prompt for a Polkit password. To avoid this, you can install a **udev rule** for your specific device.

The example below is for the **EPZ TP35 Pro**. If you are using a different device, ensure you update the `idVendor` and `idProduct` values to match your DAC (you can find these in the application's header or by running `lsusb`).

```bash
# Example for EPZ TP35 Pro (VID: 3302, PID: 43e6)
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", MODE="0666"' \
| sudo tee /etc/udev/rules.d/99-frosttune.rules

# Reload rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## Architecture

### Directory Structure
```
├── assets/           # UI icons, fonts, and screenshots
├── src/
│   ├── hardware/     # Low-level HID communication & protocols
│   │   ├── worker/   # Background thread for USB I/O
│   │   └── protocol.rs # Trait for hardware abstraction
│   ├── models/       # Data structures (Filters, Devices, IPC)
│   ├── ui/           # Iced GUI implementation
│   │   ├── views/    # Modular UI components
│   │   └── update/   # Message handling logic
│   ├── autoeq.rs     # AutoEQ format parser/generator
│   ├── storage.rs    # Profile & preference persistence
│   └── main.rs       # Entry point & helper server
├── tests/            # Integration & protocol tests
└── packaging/        # OS-specific build configurations
```

### Request Lifecycle
1. **UI Interaction**: User moves a slider or inputs a value.
2. **Message Dispatch**: A `Message` is sent to the `update` logic.
3. **Command Queue**: The UI thread sends a request to the `UsbWorker` thread via an MPSC channel.
4. **Hardware I/O**: The worker translates the request into HID packets using the specific `DeviceProtocol`.
5. **Verification**: After writing, the worker immediately reads the state back and compares it.
6. **State Sync**: The worker sends an `OperationResult` back to the UI to update the connection status.

### Contributor Guide: Adding New Devices
To support a new DAC:
1.  **Implement Protocol**: Create a new struct in `src/hardware/protocol.rs` implementing the `DeviceProtocol` trait.
2.  **Register Device**: Add your device details to the `define_devices!` macro in `src/models/device.rs`.
3.  **Test**: Add integration tests in `tests/protocol.rs` to verify packet generation.

---

## Available Scripts

| Command | Description |
|---------|-------------|
| `cargo run` | Start the application in debug mode |
| `cargo build --release` | Compile a production binary |
| `cargo test` | Run the full test suite |
| `cargo fmt` | Format the codebase |
| `makepkg -si` | (Arch Linux) Build and install from source |

---

## Testing

Frost-Tune includes a comprehensive test suite covering protocol correctness, IPC reliability, and UI state consistency.

```bash
# Run all unit and integration tests
cargo test
```

**Key Test Suites:**
- `tests/protocol.rs`: Verifies HID packet construction matches hardware expectations.
- `tests/worker_ipc.rs`: Ensures reliable communication between UI and hardware threads.
- `src/autoeq.rs`: Tests parser compatibility with various AutoEQ file formats.

---

## Packaging & Deployment

Frost-Tune is packaged for multiple platforms via GitHub Actions:
- **Arch Linux**: `PKGBUILD` provided in `packaging/arch/` for manual installation.
- **Windows**: MSI installer generated using WiX.
- **Debian/Ubuntu**: `.deb` packages available in Releases.
- **Fedora/RHEL**: `.rpm` packages available in Releases.

### Arch Linux Manual Install
```bash
cd packaging/arch
makepkg -si
```

---

## Troubleshooting

### Device Not Detected
- Ensure the device is a supported model (currently EPZ TP35 Pro).
- Check USB cable and connection.
- (Linux) Ensure your user has permissions or Polkit is installed.

### Verification Failures
- If "Write" fails with a mismatch, try "Read" first to sync the UI with current hardware state.
- Some devices may require a small delay between writes; check the `ReadTiming` in the protocol implementation.

---

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Acknowledgments

- [Iced](https://iced.rs/) — The native Rust GUI framework.
- [hidapi](https://github.com/libusb/hidapi) — Cross-platform HID communication.
- [tp35pro-eq](https://github.com/Bukutsu/tp35pro-eq) — Protocol reference.
