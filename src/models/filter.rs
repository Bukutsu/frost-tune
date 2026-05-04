use crate::models::constants::*;
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

    /// Short 2-letter label for compact UI buttons.
    pub fn short_label(&self) -> &'static str {
        match self {
            FilterType::Peak => "PK",
            FilterType::LowShelf => "LS",
            FilterType::HighShelf => "HS",
            FilterType::HighPass => "HP",
            FilterType::LowPass => "LP",
        }
    }
}

impl From<u8> for FilterType {
    fn from(value: u8) -> Self {
        match value {
            1 => FilterType::LowShelf,
            2 => FilterType::Peak,
            3 => FilterType::HighShelf,
            4 => FilterType::HighPass,
            5 => FilterType::LowPass,
            _ => FilterType::Peak,
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

impl Filter {
    pub fn enabled(index: u8, enabled: bool) -> Self {
        Filter {
            index,
            enabled,
            freq: 100,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Peak,
        }
    }

    pub fn clamp(&mut self, freq_range: (u16, u16), gain_range: (f64, f64), q_range: (f64, f64)) {
        if self.gain > gain_range.1 {
            self.gain = gain_range.1;
        } else if self.gain < gain_range.0 {
            self.gain = gain_range.0;
        }
        if self.q < q_range.0 {
            self.q = q_range.0;
        } else if self.q > q_range.1 {
            self.q = q_range.1;
        }
        if self.freq < freq_range.0 {
            self.freq = freq_range.0;
        } else if self.freq > freq_range.1 {
            self.freq = freq_range.1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PEQData {
    pub filters: Vec<Filter>,
    #[serde(rename = "globalGain")]
    pub global_gain: i8,
}

pub fn snap_freq_to_iso(freq: u16) -> u16 {
    ISO_FREQUENCIES
        .iter()
        .min_by_key(|&&f| (f as i32 - freq as i32).abs())
        .copied()
        .unwrap_or(freq.min(MAX_FREQ).max(MIN_FREQ))
}

pub fn snap_q_to_iso(q: f64) -> f64 {
    ISO_Q_VALUES
        .iter()
        .min_by_key(|&&v| ((v - q) * 100.0).abs() as i32)
        .copied()
        .unwrap_or(q.clamp(MIN_Q, MAX_Q))
}

pub fn snap_gain_step(gain: f64) -> f64 {
    (gain / GAIN_STEP).round() * GAIN_STEP
}
