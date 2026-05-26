// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

pub mod device_io;
/// # Hardware Interaction Layer
///
/// This module provides the infrastructure for discovering, authenticating, and communicating
/// with USB DAC devices. It is designed to be extensible to support multiple DAC models.
///
/// ## Adding Support for a New Device
/// The hardware layer abstracts device-specific payloads via the `DeviceProtocol` trait,
/// which lives in `crate::core::device::protocol`. To add a new device:
/// 1. Implement `crate::core::device::protocol::DeviceProtocol` for your device struct.
/// 2. Implement `crate::core::device::profile::DeviceProfile` to declare VID/PID/capabilities.
/// 3. Register the profile in `crate::hardware::registry::REGISTRY`.
/// 4. The HID layer and pipeline are device-agnostic; no changes needed there.
pub mod devices;
pub mod dsp;
#[cfg(target_os = "linux")]
pub mod elevated_transport;
#[cfg(target_os = "linux")]
pub mod helper_ipc;
#[cfg(target_os = "linux")]
pub mod helper_server;
pub mod hid;
pub mod ipc_types;
pub mod operations;
pub mod packet_builder;
pub mod pipeline;
pub mod registry;
pub mod transport;
pub mod worker;

pub use device_io::*;
pub use dsp::*;
pub use ipc_types::*;
pub use registry::*;
pub use transport::*;
pub use worker::*;
