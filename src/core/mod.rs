// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Core domain logic independent of UI, persistence, or hardware specifics.
//!
//! This module contains the business logic for EQ filtering and device protocols.
//! It has **zero dependencies** on the UI layer, allowing for reuse in CLI tools,
//! mobile apps, or headless services.

pub mod device;
pub mod eq;
pub mod ipc;

pub use device::{
    get_profile, list_profiles, DeviceCapabilities, DeviceInfo, DeviceProfile, FilterTypeFlags,
    REGISTRY,
};
pub use eq::constants::*;
pub use eq::{snap_freq_to_iso, snap_gain_step, snap_q_to_iso, Filter, FilterType, PEQData};
pub use ipc::*;
