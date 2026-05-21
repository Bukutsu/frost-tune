// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Core domain logic independent of UI, persistence, or hardware specifics.
//!
//! This module contains the business logic for EQ filtering and device protocols.
//! It has **zero dependencies** on the UI layer, allowing for reuse in CLI tools,
//! mobile apps, or headless services.

pub mod device;
pub mod eq;

pub use device::Device;
pub use eq::{Filter, FilterType, PEQData};
