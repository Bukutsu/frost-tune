use crate::diagnostics::DiagnosticsStore;
use crate::hardware::worker::UsbWorker;
use crate::models::{DeviceInfo, Filter};
use crate::storage::Profile;
use crate::ui::messages::StatusMessage;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisconnectReason {
    None,
    Manual,
    DeviceLost,
    Error(String),
}

impl Default for DisconnectReason {
    fn default() -> Self {
        DisconnectReason::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputBuffer {
    pub active_draft: Option<DraftFilter>,
}

impl InputBuffer {
    pub fn get_freq_input(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .map(|d| d.freq_input.as_str())
    }

    pub fn get_gain_input(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .map(|d| d.gain_input.as_str())
    }

    pub fn get_q_input(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .map(|d| d.q_input.as_str())
    }

    pub fn get_freq_error(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .and_then(|d| d.freq_error.as_ref().map(|s| s.as_str()))
    }

    pub fn get_gain_error(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .and_then(|d| d.gain_error.as_ref().map(|s| s.as_str()))
    }

    pub fn get_q_error(&self, band_index: usize) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .and_then(|d| d.q_error.as_ref().map(|s| s.as_str()))
    }

    pub fn has_errors(&self) -> bool {
        self.active_draft.as_ref().map_or(false, |d| d.has_errors())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DraftFilter {
    pub index: usize,
    pub freq_input: String,
    pub gain_input: String,
    pub q_input: String,
    pub freq_error: Option<String>,
    pub gain_error: Option<String>,
    pub q_error: Option<String>,
}

impl DraftFilter {
    pub fn from_filter(filter: &Filter) -> Self {
        Self {
            index: filter.index as usize,
            freq_input: filter.freq.to_string(),
            gain_input: format!("{:.1}", filter.gain), // Format to 1 decimal place
            q_input: format!("{:.2}", filter.q),       // Format to 2 decimal places
            freq_error: None,
            gain_error: None,
            q_error: None,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.freq_error.is_some() || self.gain_error.is_some() || self.q_error.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfirmAction {
    #[default]
    None,
    ResetFilters,
    DeleteProfile,
    ElevatedConnect(DeviceInfo),
    ImportAutoEQ(crate::models::PEQData),
    PullDevice,
}

#[derive(Debug, Clone, Default)]
pub struct EditorState {
    pub filters: Vec<Filter>,
    pub global_gain: i8,
    pub status_message: Option<StatusMessage>,
    pub diagnostics_errors_only: bool,
    pub profiles: Vec<Profile>,
    pub selected_profile_name: Option<String>,
    pub new_profile_name: String,
    pub input_buffer: InputBuffer,
    pub pending_confirm: ConfirmAction,
    pub profiles_dir_mtime: Option<std::time::SystemTime>,
    pub is_dirty: bool,
    pub is_autoeq_active: bool,
    pub show_diagnostics: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OperationLock {
    pub is_pulling: bool,
    pub is_pushing: bool,
    pub is_connecting: bool,
    pub is_disconnecting: bool,
}

#[derive(Default)]
pub struct MainWindow {
    pub connection_status: ConnectionStatus,
    pub disconnect_reason: DisconnectReason,
    pub editor_state: EditorState,
    pub operation_lock: OperationLock,
    pub worker: Option<Arc<UsbWorker>>,
    pub diagnostics: DiagnosticsStore,
    pub connected_device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
    pub selected_device_index: Option<usize>,
    pub connection_generation: u64,
    pub suspend_status_polling: bool,
}
