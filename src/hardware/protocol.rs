// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Re-exports of the device protocol trait and implementations from `core/`.
//!
//! Hardware protocol implementations now live in `core/device/tp35pro.rs` so that
//! `core/` is self-contained. This module provides backward-compatible re-exports
//! so the rest of `hardware/` can keep its existing import paths.

pub use crate::core::device::protocol::DeviceProtocol;
pub use crate::core::device::tp35pro::{
    TP35ProProtocol, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, READ, WRITE,
};
