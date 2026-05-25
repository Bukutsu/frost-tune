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
    pub fn clamp(
        &mut self,
        freq_range: (u16, u16),
        gain_range: (f64, f64),
        q_range: (f64, f64),
        global_gain_range: (i8, i8),
    ) {
        for filter in &mut self.filters {
            filter.clamp(freq_range, gain_range, q_range);
        }
        if let Some(gain) = self.global_gain {
            self.global_gain = Some(gain.clamp(global_gain_range.0, global_gain_range.1));
        }
    }

    pub fn is_valid(
        &self,
        num_bands: usize,
        freq_range: (u16, u16),
        gain_range: (f64, f64),
        q_range: (f64, f64),
        global_gain_range: (i8, i8),
    ) -> Result<(), String> {
        if self.filters.len() != num_bands {
            return Err(format!(
                "Expected {} filters, got {}",
                num_bands,
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
            if f.freq < freq_range.0 || f.freq > freq_range.1 {
                return Err(format!("Band {} freq out of range: {}", f.index, f.freq));
            }
            if f.gain < gain_range.0 || f.gain > gain_range.1 {
                return Err(format!("Band {} gain out of range: {}", f.index, f.gain));
            }
            if f.q < q_range.0 || f.q > q_range.1 {
                return Err(format!("Band {} Q out of range: {}", f.index, f.q));
            }
        }
        if let Some(gain) = self.global_gain {
            if !(global_gain_range.0..=global_gain_range.1).contains(&gain) {
                return Err(format!("Global gain out of range: {}", gain));
            }
        }
        Ok(())
    }
}
