use crate::core::DeviceProfile;
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
    /// Resolves the currently connected device profile, or `None` if none.
    pub fn active_device(&self) -> Option<&'static dyn DeviceProfile> {
        self.connection
            .connected_device
            .as_ref()
            .and_then(|info| crate::core::device::get_profile(info.vendor_id, info.product_id))
    }

    /// Returns the global gain range for the currently connected device.
    pub fn global_gain_range(&self) -> std::ops::RangeInclusive<i8> {
        if let Some(profile) = self.active_device() {
            let caps = profile.capabilities();
            caps.global_gain_range.0..=caps.global_gain_range.1
        } else {
            crate::core::MIN_GLOBAL_GAIN..=crate::core::MAX_GLOBAL_GAIN
        }
    }
}
