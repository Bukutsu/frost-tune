// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Re-exports of the device protocol trait and implementations.
//!
//! Hardware protocol implementations now live in `hardware/devices/tp35pro.rs`.
//! This module provides backward-compatible re-exports
//! so the rest of `hardware/` can keep its existing import paths.

pub use crate::core::device::protocol::DeviceProtocol;
pub use crate::hardware::devices::tp35pro::{
    TP35ProProtocol, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, READ, WRITE,
};
