// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::Device;
use crate::diagnostics::DiagnosticsStore;
use crate::ui::components::connection::ConnectionComponent;
use crate::ui::components::editor::EditorComponent;

#[derive(Default)]
pub struct MainWindow {
    pub connection: ConnectionComponent,
    pub editor: EditorComponent,
    pub diagnostics: DiagnosticsStore,
    pub connection_generation: u64,
    pub suspend_status_polling: bool,
    pub last_auto_reconnect_attempt: Option<std::time::Instant>,
    pub auto_reconnect_attempts: u32,
    pub last_profile_check: Option<std::time::Instant>,
}

impl MainWindow {
    /// Resolves the currently connected device, or `Device::Unknown` if none.
    pub fn active_device(&self) -> Device {
        self.connection
            .connected_device
            .as_ref()
            .map(|info| Device::from_vid_pid(info.vendor_id, info.product_id))
            .unwrap_or(Device::Unknown)
    }

    /// Returns the global gain range for the currently connected device.
    pub fn global_gain_range(&self) -> std::ops::RangeInclusive<i8> {
        let dev = self.active_device();
        dev.min_global_gain()..=dev.max_global_gain()
    }
}
