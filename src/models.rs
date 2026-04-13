use serde::{Deserialize, Serialize};

const TP35_VENDOR_ID: u16 = 0x3302;
const TP35_PRODUCT_ID: u16 = 0x43E6;

pub const VENDOR_ID: u16 = TP35_VENDOR_ID;
pub const PRODUCT_ID: u16 = TP35_PRODUCT_ID;

pub const MAX_BAND_GAIN: f64 = 10.0;
pub const MIN_BAND_GAIN: f64 = -10.0;
pub const MAX_GLOBAL_GAIN: i8 = 10;
pub const MIN_GLOBAL_GAIN: i8 = -10;
pub const NUM_BANDS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    TP35Pro,
    Unknown,
}

impl Device {
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
        if self.q < 0.1 {
            self.q = 0.1;
        } else if self.q > 20.0 {
            self.q = 20.0;
        }
        if self.freq < 20 {
            self.freq = 20;
        } else if self.freq > 20000 {
            self.freq = 20000;
        }
    }
}

impl PushPayload {
    pub fn clamp(&mut self) {
        for filter in &mut self.filters {
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
            return Err(format!("Expected {} filters, got {}", NUM_BANDS, self.filters.len()));
        }
        for f in &self.filters {
            if f.freq < 20 || f.freq > 20000 {
                return Err(format!("Band {} freq out of range: {}", f.index, f.freq));
            }
            if f.gain < MIN_BAND_GAIN || f.gain > MAX_BAND_GAIN {
                return Err(format!("Band {} gain out of range: {}", f.index, f.gain));
            }
            if f.q < 0.1 || f.q > 20.0 {
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
        let mut payload = PushPayload { filters: vec![], global_gain: Some(15) };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MAX_GLOBAL_GAIN));
    }

    #[test]
    fn test_global_gain_clamp_min() {
        let mut payload = PushPayload { filters: vec![], global_gain: Some(-15) };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MIN_GLOBAL_GAIN));
    }

    #[test]
    fn test_push_payload_valid_with_10_bands() {
        let filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let payload = PushPayload { filters: filters.clone(), global_gain: Some(5) };
        assert!(payload.is_valid().is_ok());
    }

    #[test]
    fn test_push_payload_invalid_with_wrong_band_count() {
        let filters = vec![Filter::enabled(0, false)];
        let payload = PushPayload { filters, global_gain: Some(0) };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_default_filter_has_correct_index() {
        for i in 0..10 {
            let filter = Filter::enabled(i, true);
            assert_eq!(filter.index, i as u8);
        }
    }
}
