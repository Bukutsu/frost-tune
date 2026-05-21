// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Device abstraction: supported devices, their protocols, and device discovery.
//!
//! This module defines the interface for hardware devices and the registry of supported
//! USB DACs. It is UI-agnostic and provides a clean API for device communication.

#[allow(clippy::module_inception)]
pub mod device;
pub mod protocol;

pub use device::{Device, DeviceInfo};
pub use protocol::DeviceProtocol;
