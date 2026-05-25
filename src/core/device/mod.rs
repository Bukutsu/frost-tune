// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Device abstraction: supported devices, their protocols, and device discovery.
//!
//! This module defines the interface for hardware devices and the registry of supported
//! USB DACs. It is UI-agnostic and provides a clean API for device communication.

pub mod capabilities;
#[allow(clippy::module_inception)]
pub mod device;
pub mod protocol;
pub mod registry;
pub mod timing;
pub mod tp35pro;

pub use capabilities::{DeviceCapabilities, FilterTypeFlags};
pub use device::DeviceInfo;
pub use protocol::DeviceProtocol;
pub use registry::{get_profile, list_profiles, DeviceProfile, REGISTRY};
pub use timing::{ReadTiming, WriteTiming};
