/// # Hardware Interaction Layer
/// 
/// This module provides the infrastructure for discovering, authenticating, and communicating 
/// with USB DAC devices. It is designed to be extensible to support multiple DAC models.
/// 
/// ## Adding Support for a New Device
/// The hardware layer abstracts device-specific payloads via the `DeviceProtocol` trait.
/// To add a new device:
/// 1. **Models**: Add the device VID/PID and enum variant in `crate::models::Device`.
/// 2. **Protocol**: Implement `crate::hardware::protocol::DeviceProtocol` for your device to define its USB packet layouts.
/// 3. **HID Layer**: The `worker.rs` and `hid.rs` modules automatically use the correct protocol implementation based on the `Device` enum mapping.
/// 4. **Elevated Transport (Linux)**: The DBus helper daemon uses `elevated_transport.rs` which works at the raw USB level and is completely device-agnostic. No changes are needed there for new devices.
pub mod dsp;
#[cfg(target_os = "linux")]
pub mod elevated_transport;
#[cfg(target_os = "linux")]
pub mod helper_ipc;
#[cfg(target_os = "linux")]
pub mod helper_server;
pub mod hid;
pub mod operations;
pub mod packet_builder;
pub mod protocol;
pub mod worker;

pub use dsp::*;
pub use protocol::*;
pub use worker::*;
