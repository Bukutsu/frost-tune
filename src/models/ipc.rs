// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind};
use crate::models::constants::*;
use crate::models::device::DeviceInfo;
use crate::models::filter::{Filter, PEQData};
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
                ErrorKind::Unknown,
                "Operation timed out after 5 seconds",
            )),
        }
    }

    pub fn worker_gone() -> Self {
        OperationResult {
            success: false,
            data: None,
            error: Some(AppError::new(
                ErrorKind::Unknown,
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
    pub fn clamp(&mut self) {
        for filter in &mut self.filters {
            filter.clamp(
                (crate::models::MIN_FREQ, crate::models::MAX_FREQ),
                (crate::models::MIN_BAND_GAIN, crate::models::MAX_BAND_GAIN),
                (crate::models::MIN_Q, crate::models::MAX_Q),
            );
        }
        if let Some(gain) = self.global_gain {
            self.global_gain = Some(gain.clamp(MIN_GLOBAL_GAIN, MAX_GLOBAL_GAIN));
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
            if !(MIN_GLOBAL_GAIN..=MAX_GLOBAL_GAIN).contains(&gain) {
                return Err(format!("Global gain out of range: {}", gain));
            }
        }
        Ok(())
    }
}
