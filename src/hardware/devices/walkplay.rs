// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Generic Walkplay DSP protocol family: profiles for Moondrop Dawn Pro and Truthear KEYX.
//!
//! Reuses the core DSP math and DeviceProtocol implementation from `walkplay_protocol.rs`
//! since they share the exact same wire format and chipset.

use super::walkplay_protocol::WalkplayProtocol;
use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::device::profile::DeviceProfile;
use crate::core::device::protocol::DeviceProtocol;
pub struct DawnProProfile;

impl DeviceProfile for DawnProProfile {
    fn name(&self) -> &'static str {
        "Moondrop Dawn Pro"
    }

    fn vendor_id(&self) -> u16 {
        0x2FC6
    }

    fn product_id(&self) -> u16 {
        0xDF30
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 8,
            global_gain_range: (-20, 0),
            band_gain_range: (-12.0, 12.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: false,
            dsp_sample_rate: 96000.0,
            gain_tolerance: 0.1,
            freq_tolerance: 1,
            q_tolerance: 0.05,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(WalkplayProtocol)
    }
}

pub struct TruthearKeyxProfile;

impl DeviceProfile for TruthearKeyxProfile {
    fn name(&self) -> &'static str {
        "Truthear KEYX"
    }

    fn vendor_id(&self) -> u16 {
        0x0D8C
    }

    fn product_id(&self) -> u16 {
        0x0210
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 8,
            global_gain_range: (-20, 0),
            band_gain_range: (-12.0, 12.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: false,
            dsp_sample_rate: 96000.0,
            gain_tolerance: 0.1,
            freq_tolerance: 1,
            q_tolerance: 0.05,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(WalkplayProtocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dawn_pro_capabilities() {
        let profile = DawnProProfile;
        let caps = profile.capabilities();
        assert_eq!(caps.num_bands, 8);
        assert_eq!(profile.vendor_id(), 0x2FC6);
        assert_eq!(profile.product_id(), 0xDF30);
    }

    #[test]
    fn test_keyx_capabilities() {
        let profile = TruthearKeyxProfile;
        let caps = profile.capabilities();
        assert_eq!(caps.num_bands, 8);
        assert_eq!(profile.vendor_id(), 0x0D8C);
        assert_eq!(profile.product_id(), 0x0210);
    }
}
