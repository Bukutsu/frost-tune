// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::eq::constants::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterType {
    #[serde(rename = "LSQ")]
    LowShelf = 1,
    #[serde(rename = "PK")]
    Peak = 2,
    #[serde(rename = "HSQ")]
    HighShelf = 3,
    #[serde(rename = "HP")]
    HighPass = 4,
    #[serde(rename = "LP")]
    LowPass = 5,
}

impl FilterType {
    /// All supported filter types in UI display order.
    pub const ALL: &[FilterType] = &[
        FilterType::Peak,
        FilterType::HighShelf,
        FilterType::LowShelf,
        FilterType::HighPass,
        FilterType::LowPass,
    ];
}

impl From<u8> for FilterType {
    fn from(value: u8) -> Self {
        match value {
            1 => FilterType::LowShelf,
            2 => FilterType::Peak,
            3 => FilterType::HighShelf,
            4 => FilterType::HighPass,
            5 => FilterType::LowPass,
            _ => {
                log::warn!(
                    "Unknown FilterType byte {:#04x} in device response — defaulting to Peak. \
                     Your device likely uses a different filter-type encoding; see CONTRIBUTING_DEVICES.md.",
                    value
                );
                FilterType::Peak
            }
        }
    }
}

impl From<FilterType> for u8 {
    fn from(ft: FilterType) -> Self {
        ft as u8
    }
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterType::LowShelf => write!(f, "Low Shelf"),
            FilterType::Peak => write!(f, "Peak"),
            FilterType::HighShelf => write!(f, "High Shelf"),
            FilterType::HighPass => write!(f, "High Pass"),
            FilterType::LowPass => write!(f, "Low Pass"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    pub index: u8,
    pub enabled: bool,
    pub freq: u16,
    pub gain: f64,
    pub q: f64,
    #[serde(rename = "type")]
    pub filter_type: FilterType,
}

const DEFAULT_FREQS_10_BAND: [u16; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

impl Filter {
    pub fn enabled(index: u8, enabled: bool) -> Self {
        let freq = DEFAULT_FREQS_10_BAND
            .get(index as usize)
            .copied()
            .unwrap_or(1000);
        Filter {
            index,
            enabled,
            freq,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Peak,
        }
    }

    pub fn clamp(&mut self, freq_range: (u16, u16), gain_range: (f64, f64), q_range: (f64, f64)) {
        self.gain = self.gain.clamp(gain_range.0, gain_range.1);
        self.q = self.q.clamp(q_range.0, q_range.1);
        self.freq = self.freq.clamp(freq_range.0, freq_range.1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PEQData {
    pub filters: Vec<Filter>,
    #[serde(rename = "globalGain")]
    pub global_gain: i8,
}

use crate::core::device::capabilities::DeviceCapabilities;

impl PEQData {
    /// Clamps the EQ data to fit within the given device capabilities.
    pub fn clamp_to_capabilities(&mut self, caps: &DeviceCapabilities) {
        self.global_gain = self
            .global_gain
            .clamp(caps.global_gain_range.0, caps.global_gain_range.1);

        // Truncate if there are more filters than supported bands
        if self.filters.len() > caps.num_bands {
            self.filters.truncate(caps.num_bands);
        }

        // Pad with disabled filters if there are fewer filters than supported bands
        while self.filters.len() < caps.num_bands {
            self.filters
                .push(Filter::enabled(self.filters.len() as u8, false));
        }

        for filter in &mut self.filters {
            filter.clamp(caps.freq_range, caps.band_gain_range, caps.q_range);
            if !caps.supported_filter_types.supports(filter.filter_type) {
                filter.filter_type = FilterType::Peak; // Fallback
            }
            if !caps.supports_per_band_enable {
                // If per-band enable is not supported, effectively disable by zeroing gain
                if !filter.enabled {
                    filter.gain = 0.0;
                }
            }
        }
    }
    /// Audibly-equivalent comparison with tolerance for float fields.
    /// Disabled bands match regardless of params (no audible effect).
    pub fn matches_within(&self, other: &Self, gain_tol: f64, q_tol: f64) -> bool {
        if self.global_gain != other.global_gain {
            return false;
        }
        if self.filters.len() != other.filters.len() {
            return false;
        }
        self.filters
            .iter()
            .zip(other.filters.iter())
            .all(|(a, b)| filter_matches_within(a, b, gain_tol, q_tol))
    }
}

fn filter_matches_within(a: &Filter, b: &Filter, gain_tol: f64, q_tol: f64) -> bool {
    if a.enabled != b.enabled {
        return false;
    }
    if !a.enabled {
        return true;
    }
    a.filter_type == b.filter_type
        && a.freq == b.freq
        && (a.gain - b.gain).abs() <= gain_tol
        && (a.q - b.q).abs() <= q_tol
}

pub fn snap_freq_to_iso(freq: u16) -> u16 {
    let idx = ISO_FREQUENCIES.partition_point(|&f| f < freq);
    if idx == 0 {
        ISO_FREQUENCIES[0]
    } else if idx >= ISO_FREQUENCIES.len() {
        ISO_FREQUENCIES[ISO_FREQUENCIES.len() - 1]
    } else {
        let left = ISO_FREQUENCIES[idx - 1];
        let right = ISO_FREQUENCIES[idx];
        if (freq - left) <= (right - freq) {
            left
        } else {
            right
        }
    }
}

pub fn snap_q_to_iso(q: f64) -> f64 {
    let idx = ISO_Q_VALUES.partition_point(|&v| v < q);
    if idx == 0 {
        ISO_Q_VALUES[0]
    } else if idx >= ISO_Q_VALUES.len() {
        ISO_Q_VALUES[ISO_Q_VALUES.len() - 1]
    } else {
        let left = ISO_Q_VALUES[idx - 1];
        let right = ISO_Q_VALUES[idx];
        if (q - left) <= (right - q) {
            left
        } else {
            right
        }
    }
}

pub fn snap_gain_step(gain: f64) -> f64 {
    (gain / GAIN_STEP).round() * GAIN_STEP
}
