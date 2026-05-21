// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Service layer facades: encapsulates background workers and storage.

pub mod hardware;
pub mod preset;

pub use hardware::HardwareService;
pub use preset::PresetService;

// Re-export domain types from core/ for unified access by callers
pub use crate::core::device::{Device, DeviceInfo};
pub use crate::core::eq::{
    snap_freq_to_iso, snap_gain_step, snap_q_to_iso, Filter, FilterType, PEQData,
};
