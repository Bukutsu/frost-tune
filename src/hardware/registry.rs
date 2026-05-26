// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Static registry of compiled-in USB DAC profiles.
//!
//! The `DeviceProfile` trait lives in `crate::core::device::profile`. This module
//! owns the concrete registry array and the VID/PID lookup helpers.

pub use crate::core::device::profile::DeviceProfile;

/// All supported USB DAC profiles compiled into the binary.
///
/// To add a new device: implement `DeviceProfile` in `hardware/devices/<vendor>/mod.rs`
/// and add a `&<YourProfile>` entry here.
pub const REGISTRY: &[&dyn DeviceProfile] = &[
    &crate::hardware::devices::tp35pro::TP35ProProfile,
    &crate::hardware::devices::walkplay::DawnProProfile,
    &crate::hardware::devices::walkplay::TruthearKeyxProfile,
];

/// Resolves a `DeviceProfile` from a USB Vendor ID and Product ID.
pub fn get_profile(vid: u16, pid: u16) -> Option<&'static dyn DeviceProfile> {
    REGISTRY
        .iter()
        .find(|p| p.vendor_id() == vid && p.product_id() == pid)
        .copied()
}

/// Lists all supported devices in the registry.
pub fn list_profiles() -> &'static [&'static dyn DeviceProfile] {
    REGISTRY
}
