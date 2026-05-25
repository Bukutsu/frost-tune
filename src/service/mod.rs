// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Service layer facades: encapsulates background workers and storage.

pub mod hardware;
pub mod preset;

pub use hardware::HardwareService;
pub use preset::PresetService;
