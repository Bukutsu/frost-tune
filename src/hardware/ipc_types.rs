// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::device::DeviceInfo;

use crate::core::eq::{Filter, PEQData};
use crate::error::{AppError, ErrorKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub device: Option<DeviceInfo>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub data: Option<PEQData>,
    pub error: Option<AppError>,
}

impl OperationResult {
    pub fn timed_out() -> Self {
        OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(
                ErrorKind::Timeout,
                "Operation timed out after 5 seconds",
            )),
        }
    }

    pub fn worker_gone() -> Self {
        OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(
                ErrorKind::WorkerDied,
                "Background worker unexpectedly terminated",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    pub filters: Vec<Filter>,
    pub global_gain: Option<i8>,
}

impl PushPayload {
    pub fn new_validated(
        filters: Vec<Filter>,
        global_gain: Option<i8>,
        caps: &crate::core::device::DeviceCapabilities,
    ) -> Result<Self, String> {
        let mut payload = Self {
            filters,
            global_gain,
        };
        payload.clamp(caps);
        payload.is_valid(caps)?;
        Ok(payload)
    }

    pub fn clamp(&mut self, caps: &crate::core::device::DeviceCapabilities) {
        for filter in &mut self.filters {
            filter.clamp(caps.freq_range, caps.band_gain_range, caps.q_range);
            if !caps.supports_per_band_enable && !filter.enabled {
                // Hardware without a per-band enable bit cannot represent an
                // off slot. Write an audibly-neutral band instead so pushing a
                // preset with disabled bands does not fail validation.
                filter.gain = 0.0;
            }
        }
        if let Some(gain) = self.global_gain {
            self.global_gain = Some(gain.clamp(caps.global_gain_range.0, caps.global_gain_range.1));
        }
    }

    pub fn is_valid(&self, caps: &crate::core::device::DeviceCapabilities) -> Result<(), String> {
        if self.filters.len() != caps.num_bands {
            return Err(format!(
                "Expected {} filters, got {}",
                caps.num_bands,
                self.filters.len()
            ));
        }
        for f in &self.filters {
            if !f.gain.is_finite() {
                return Err(format!("Band {} gain is not a finite number", f.index));
            }
            if !f.q.is_finite() {
                return Err(format!("Band {} Q is not a finite number", f.index));
            }
            if f.freq < caps.freq_range.0 || f.freq > caps.freq_range.1 {
                return Err(format!("Band {} freq out of range: {}", f.index, f.freq));
            }
            if f.gain < caps.band_gain_range.0 || f.gain > caps.band_gain_range.1 {
                return Err(format!("Band {} gain out of range: {}", f.index, f.gain));
            }
            if f.q < caps.q_range.0 || f.q > caps.q_range.1 {
                return Err(format!("Band {} Q out of range: {}", f.index, f.q));
            }
            if !caps.supported_filter_types.supports(f.filter_type) {
                return Err(format!(
                    "Band {} uses unsupported filter type: {:?}",
                    f.index, f.filter_type
                ));
            }
            if !caps.supports_per_band_enable && !f.enabled && f.gain.abs() > 0.001 {
                return Err(format!(
                    "Band {} cannot be disabled with non-zero gain on this hardware",
                    f.index
                ));
            }
        }
        if let Some(gain) = self.global_gain {
            if !(caps.global_gain_range.0..=caps.global_gain_range.1).contains(&gain) {
                return Err(format!("Global gain out of range: {}", gain));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::device::capabilities::{DeviceCapabilities, DESKTOP_DAC_CAPS};
    use crate::core::eq::Filter;

    fn caps_without_per_band_enable(num_bands: usize) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands,
            supports_per_band_enable: false,
            ..DESKTOP_DAC_CAPS
        }
    }

    #[test]
    fn new_validated_zeroes_disabled_band_gain_when_hardware_lacks_enable_bit() {
        let caps = caps_without_per_band_enable(1);
        let mut filter = Filter::enabled(0, false);
        filter.gain = 4.5;

        let payload = PushPayload::new_validated(vec![filter], Some(0), &caps)
            .expect("disabled bands should be represented as neutral writes");

        assert!(!payload.filters[0].enabled);
        assert_eq!(payload.filters[0].gain, 0.0);
    }

    #[test]
    fn is_valid_accepts_neutral_disabled_band_when_hardware_lacks_enable_bit() {
        let caps = caps_without_per_band_enable(1);
        let filter = Filter::enabled(0, false);

        let payload = PushPayload {
            filters: vec![filter],
            global_gain: Some(0),
        };

        assert!(payload.is_valid(&caps).is_ok());
    }
}
