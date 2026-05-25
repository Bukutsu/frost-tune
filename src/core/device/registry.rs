// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Dynamic Device Registry system replacing the hardcoded Device enum.

use crate::core::device::capabilities::DeviceCapabilities;
use crate::core::device::protocol::DeviceProtocol;

/// Interface representing a supported USB DAC model profile.
pub trait DeviceProfile: Send + Sync {
    /// The friendly name of the DAC model.
    fn name(&self) -> &'static str;

    /// The USB Vendor ID.
    fn vendor_id(&self) -> u16;

    /// The USB Product ID.
    fn product_id(&self) -> u16;

    /// The capability boundaries for EQ frequency, gain, and Q.
    fn capabilities(&self) -> DeviceCapabilities;

    /// Factory method instantiating the communication protocol for this DAC.
    fn protocol(&self) -> Box<dyn DeviceProtocol>;
}

/// The static registry containing all compiled-in USB DAC profiles.
///
/// Under the Open-Closed Principle, adding a new device only requires implementing
/// `DeviceProfile` and adding its static reference to this array.
pub const REGISTRY: &[&dyn DeviceProfile] = &[&crate::core::device::tp35pro::TP35ProProfile];

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
