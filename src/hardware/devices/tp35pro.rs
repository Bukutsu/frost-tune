// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use super::walkplay_protocol::WalkplayProtocol;
use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::device::profile::DeviceProfile;
use crate::core::device::protocol::DeviceProtocol;

pub struct TP35ProProfile;

impl DeviceProfile for TP35ProProfile {
    fn name(&self) -> &'static str {
        "EPZ TP35 Pro"
    }

    fn vendor_id(&self) -> u16 {
        0x3302
    }

    fn product_id(&self) -> u16 {
        0x43E6
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 10,
            global_gain_range: (-16, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF
                | FilterTypeFlags::LOW_PASS
                | FilterTypeFlags::HIGH_PASS,
            supports_per_band_enable: false,
            dsp_sample_rate: 96000.0,
            gain_tolerance: 0.15,
            freq_tolerance: 1,
            q_tolerance: 0.05,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(WalkplayProtocol)
    }
}
