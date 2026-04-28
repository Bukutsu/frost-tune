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
    pub editing_freq: Option<(usize, String)>,
    pub editing_gain: Option<(usize, String)>,
    pub editing_q: Option<(usize, String)>,
    pub freq_error: Option<(usize, String)>,
    pub gain_error: Option<(usize, String)>,
    pub q_error: Option<(usize, String)>,
}

impl InputBuffer {
    pub fn get_freq(&self, band_index: usize) -> Option<&String> {
        self.editing_freq
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn get_gain(&self, band_index: usize) -> Option<&String> {
        self.editing_gain
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn get_q(&self, band_index: usize) -> Option<&String> {
        self.editing_q
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn get_freq_error(&self, band_index: usize) -> Option<&String> {
        self.freq_error
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn get_gain_error(&self, band_index: usize) -> Option<&String> {
        self.gain_error
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn get_q_error(&self, band_index: usize) -> Option<&String> {
        self.q_error
            .as_ref()
            .filter(|(i, _)| *i == band_index)
            .map(|(_, s)| s)
    }

    pub fn clear_error(&mut self, band_index: usize) {
        if let Some((i, _)) = self.freq_error.take() {
            if i != band_index {
                self.freq_error = Some((i, String::new()));
            }
        }
        if let Some((i, _)) = self.gain_error.take() {
            if i != band_index {
                self.gain_error = Some((i, String::new()));
            }
        }
        if let Some((i, _)) = self.q_error.take() {
            if i != band_index {
                self.q_error = Some((i, String::new()));
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfirmAction {
    #[default]
    None,
    ResetFilters,
    DeleteProfile,
    ElevatedConnect(DeviceInfo),
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
}
