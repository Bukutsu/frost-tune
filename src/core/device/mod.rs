// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Device abstraction: supported devices, their protocols, and device discovery.
//!
//! This module defines the interface for hardware devices and the registry of supported
//! USB DACs. It is UI-agnostic and provides a clean API for device communication.

pub mod capabilities;
#[allow(clippy::module_inception)]
pub mod device;
pub mod profile;
pub mod protocol;
pub mod timing;

pub use capabilities::{DeviceCapabilities, FilterTypeFlags};
pub use device::DeviceInfo;
pub use profile::DeviceProfile;
pub use protocol::DeviceProtocol;
pub use timing::{ReadTiming, WriteTiming};
