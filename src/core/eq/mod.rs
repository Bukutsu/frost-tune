// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! EQ domain: filters, presets, and standard EQ parameters.
//!
//! This module contains the core data structures and logic for managing parametric EQ settings.
//! It is UI-agnostic and can be used in any context (desktop, mobile, CLI, backend service).

pub mod constants;
pub mod filter;
pub mod iir_math;

pub use constants::*;
pub use filter::{snap_freq_to_iso, snap_gain_step, snap_q_to_iso, Filter, FilterType, PEQData};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_gain_clamp_at_max() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 15.0;
        filter.clamp(
            (MIN_FREQ, MAX_FREQ),
            (MIN_BAND_GAIN, MAX_BAND_GAIN),
            (MIN_Q, MAX_Q),
        );
        assert_eq!(filter.gain, MAX_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_clamp_at_min() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = -15.0;
        filter.clamp(
            (MIN_FREQ, MAX_FREQ),
            (MIN_BAND_GAIN, MAX_BAND_GAIN),
            (MIN_Q, MAX_Q),
        );
        assert_eq!(filter.gain, MIN_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_unchanged_when_in_bounds() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 5.0;
        filter.clamp(
            (MIN_FREQ, MAX_FREQ),
            (MIN_BAND_GAIN, MAX_BAND_GAIN),
            (MIN_Q, MAX_Q),
        );
        assert_eq!(filter.gain, 5.0);
    }

    #[test]
    fn test_default_filter_has_correct_index() {
        for i in 0u8..10 {
            let filter = Filter::enabled(i, true);
            assert_eq!(filter.index, i);
        }
    }

    #[test]
    fn test_snap_freq_to_iso() {
        assert_eq!(snap_freq_to_iso(100), 100);
        assert_eq!(snap_freq_to_iso(101), 100);
        assert_eq!(snap_freq_to_iso(99), 100);
        assert_eq!(snap_freq_to_iso(150), 160);
        assert_eq!(snap_freq_to_iso(15), 20);
    }

    #[test]
    fn test_snap_q_to_iso() {
        assert_eq!(snap_q_to_iso(1.0), 1.0);
        assert_eq!(snap_q_to_iso(1.1), 1.0);
        assert_eq!(snap_q_to_iso(0.1), 0.1);
        assert_eq!(snap_q_to_iso(3.0), 2.0);
    }

    #[test]
    fn test_snap_gain_step() {
        assert!((snap_gain_step(1.3) - 1.3).abs() < 0.01);
        assert!((snap_gain_step(-1.3) - (-1.3)).abs() < 0.01);
        assert!((snap_gain_step(0.0) - 0.0).abs() < 0.01);
        assert!((snap_gain_step(10.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_clamp_to_capabilities_clamps_global_gain() {
        use crate::core::device::capabilities::DeviceCapabilities;

        let caps = DeviceCapabilities {
            num_bands: 10,
            global_gain_range: (-5, 5),
            ..Default::default()
        };
        let mut peq = PEQData {
            global_gain: 20,
            filters: vec![Filter::enabled(0, true)],
        };
        peq.clamp_to_capabilities(&caps);
        assert_eq!(peq.global_gain, 5);
    }

    #[test]
    fn test_clamp_to_capabilities_pads_filters() {
        use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};

        let caps = DeviceCapabilities {
            num_bands: 5,
            global_gain_range: (-16, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 20.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: true,
        };
        let mut peq = PEQData {
            global_gain: 0,
            filters: vec![Filter::enabled(0, true)],
        };
        peq.clamp_to_capabilities(&caps);
        assert_eq!(peq.filters.len(), 5, "should pad to 5 filters");
    }

    #[test]
    fn test_clamp_to_capabilities_truncates_filters() {
        use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};

        let caps = DeviceCapabilities {
            num_bands: 2,
            global_gain_range: (-16, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 20.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: true,
        };
        let mut peq = PEQData {
            global_gain: 0,
            filters: (0..5).map(|i| Filter::enabled(i, true)).collect(),
        };
        peq.clamp_to_capabilities(&caps);
        assert_eq!(peq.filters.len(), 2, "should truncate to 2 filters");
    }

    #[test]
    fn test_clamp_to_capabilities_falls_back_unsupported_type() {
        use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};

        let caps = DeviceCapabilities {
            num_bands: 1,
            global_gain_range: (-16, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 20.0),
            supported_filter_types: FilterTypeFlags::PEAK,
            supports_per_band_enable: true,
        };
        let mut peq = PEQData {
            global_gain: 0,
            filters: vec![Filter {
                index: 0,
                enabled: true,
                filter_type: FilterType::LowShelf,
                freq: 100,
                gain: 5.0,
                q: 1.0,
            }],
        };
        peq.clamp_to_capabilities(&caps);
        assert_eq!(
            peq.filters[0].filter_type,
            FilterType::Peak,
            "unsupported type should fall back to Peak"
        );
    }
}
