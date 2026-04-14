use serde::{Deserialize, Serialize};

const TP35_VENDOR_ID: u16 = 0x3302;
const TP35_PRODUCT_ID: u16 = 0x43E6;

pub const VENDOR_ID: u16 = TP35_VENDOR_ID;
pub const PRODUCT_ID: u16 = TP35_PRODUCT_ID;

pub const MAX_BAND_GAIN: f64 = 10.0;
pub const MIN_BAND_GAIN: f64 = -10.0;
pub const GAIN_STEP: f64 = 0.1;
pub const MAX_GLOBAL_GAIN: i8 = 10;
pub const MIN_GLOBAL_GAIN: i8 = -10;
pub const MIN_Q: f64 = 0.1;
pub const MAX_Q: f64 = 20.0;
pub const MIN_FREQ: u16 = 20;
pub const MAX_FREQ: u16 = 20000;
pub const NUM_BANDS: usize = 10;

pub const ISO_FREQUENCIES: [u16; 31] = [
    20, 25, 31, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630, 800, 1000, 1250, 1600,
    2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 20000,
];

pub const ISO_Q_VALUES: [f64; 10] = [0.1, 0.25, 0.5, 0.707, 1.0, 1.4, 2.0, 4.0, 8.0, 16.0];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    TP35Pro,
    Unknown,
}

use crate::hardware::protocol::{DeviceProtocol, TP35ProProtocol};

impl Device {
    pub fn protocol(&self) -> Box<dyn DeviceProtocol> {
        match self {
            Device::TP35Pro => Box::new(TP35ProProtocol),
            Device::Unknown => Box::new(TP35ProProtocol), // Fallback
        }
    }

    pub fn from_vid_pid(vid: u16, pid: u16) -> Self {
        match (vid, pid) {
            (TP35_VENDOR_ID, TP35_PRODUCT_ID) => Device::TP35Pro,
            _ => Device::Unknown,
        }
    }

    pub fn vendor_id(&self) -> u16 {
        match self {
            Device::TP35Pro => TP35_VENDOR_ID,
            Device::Unknown => 0,
        }
    }

    pub fn product_id(&self) -> u16 {
        match self {
            Device::TP35Pro => TP35_PRODUCT_ID,
            Device::Unknown => 0,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Device::TP35Pro => "Topping TP35 Pro",
            Device::Unknown => "Unknown Device",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterType {
    #[serde(rename = "LSQ")]
    LowShelf = 1,
    #[serde(rename = "PK")]
    Peak = 2,
    #[serde(rename = "HSQ")]
    HighShelf = 3,
}

impl From<u8> for FilterType {
    fn from(value: u8) -> Self {
        match value {
            1 => FilterType::LowShelf,
            2 => FilterType::Peak,
            3 => FilterType::HighShelf,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PEQData {
    pub filters: Vec<Filter>,
    #[serde(rename = "globalGain")]
    pub global_gain: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub device: Option<DeviceInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    pub filters: Vec<Filter>,
    pub global_gain: Option<i8>,
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

    pub fn clamp(&mut self) {
        if self.gain > MAX_BAND_GAIN {
            self.gain = MAX_BAND_GAIN;
        } else if self.gain < MIN_BAND_GAIN {
            self.gain = MIN_BAND_GAIN;
        }
        if self.q < MIN_Q {
            self.q = MIN_Q;
        } else if self.q > MAX_Q {
            self.q = MAX_Q;
        }
        if self.freq < MIN_FREQ {
            self.freq = MIN_FREQ;
        } else if self.freq > MAX_FREQ {
            self.freq = MAX_FREQ;
        }
    }
}

impl PushPayload {
    pub fn clamp(&mut self) {
        for filter in &mut self.filters {
            filter.enabled = true;
            filter.clamp();
        }
        if let Some(ref mut gain) = self.global_gain {
            if *gain > MAX_GLOBAL_GAIN {
                *gain = MAX_GLOBAL_GAIN;
            } else if *gain < MIN_GLOBAL_GAIN {
                *gain = MIN_GLOBAL_GAIN;
            }
        }
    }

    pub fn is_valid(&self) -> Result<(), String> {
        if self.filters.len() != NUM_BANDS {
            return Err(format!(
                "Expected {} filters, got {}",
                NUM_BANDS,
                self.filters.len()
            ));
        }
        for f in &self.filters {
            if !f.enabled {
                return Err(format!("Band {} must be enabled in push payload", f.index));
            }
            if !f.gain.is_finite() {
                return Err(format!("Band {} gain is not a finite number", f.index));
            }
            if !f.q.is_finite() {
                return Err(format!("Band {} Q is not a finite number", f.index));
            }
            if f.freq < MIN_FREQ || f.freq > MAX_FREQ {
                return Err(format!("Band {} freq out of range: {}", f.index, f.freq));
            }
            if f.gain < MIN_BAND_GAIN || f.gain > MAX_BAND_GAIN {
                return Err(format!("Band {} gain out of range: {}", f.index, f.gain));
            }
            if f.q < MIN_Q || f.q > MAX_Q {
                return Err(format!("Band {} Q out of range: {}", f.index, f.q));
            }
        }
        if let Some(gain) = self.global_gain {
            if gain < MIN_GLOBAL_GAIN || gain > MAX_GLOBAL_GAIN {
                return Err(format!("Global gain out of range: {}", gain));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_gain_clamp_at_max() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 15.0;
        filter.clamp();
        assert_eq!(filter.gain, MAX_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_clamp_at_min() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = -15.0;
        filter.clamp();
        assert_eq!(filter.gain, MIN_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_unchanged_when_in_bounds() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 5.0;
        filter.clamp();
        assert_eq!(filter.gain, 5.0);
    }

    #[test]
    fn test_global_gain_clamp_max() {
        let mut payload = PushPayload {
            filters: vec![],
            global_gain: Some(15),
        };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MAX_GLOBAL_GAIN));
    }

    #[test]
    fn test_global_gain_clamp_min() {
        let mut payload = PushPayload {
            filters: vec![],
            global_gain: Some(-15),
        };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MIN_GLOBAL_GAIN));
    }

    #[test]
    fn test_push_payload_valid_with_10_bands() {
        let filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, true)).collect();
        let payload = PushPayload {
            filters: filters.clone(),
            global_gain: Some(5),
        };
        assert!(payload.is_valid().is_ok());
    }

    #[test]
    fn test_push_payload_invalid_with_wrong_band_count() {
        let filters = vec![Filter::enabled(0, false)];
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_invalid_with_disabled_band() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, true)).collect();
        filters[2].enabled = false;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_clamp_enables_all_bands() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let mut payload = PushPayload {
            filters: std::mem::take(&mut filters),
            global_gain: Some(0),
        };
        payload.clamp();
        assert!(payload.filters.iter().all(|f| f.enabled));
    }

    #[test]
    fn test_push_payload_invalid_with_nan_gain() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        filters[0].gain = f64::NAN;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_invalid_with_inf_q() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        filters[0].q = f64::INFINITY;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_default_filter_has_correct_index() {
        for i in 0..10 {
            let filter = Filter::enabled(i, true);
            assert_eq!(filter.index, i as u8);
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
}
