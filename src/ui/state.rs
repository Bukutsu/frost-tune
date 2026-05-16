use crate::diagnostics::DiagnosticsStore;
use crate::hardware::worker::UsbWorker;
use crate::models::{DeviceInfo, Filter};
use crate::storage::Profile;
use crate::ui::messages::StatusMessage;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DisconnectReason {
    #[default]
    None,
    Manual,
    DeviceLost,
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct InputBuffer {
    pub active_draft: Option<DraftFilter>,
}

impl InputBuffer {
    fn get_input_for(
        &self,
        band_index: usize,
        f: impl FnOnce(&DraftFilter) -> &str,
    ) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .map(f)
    }

    fn get_error_for(
        &self,
        band_index: usize,
        f: impl FnOnce(&DraftFilter) -> &Option<String>,
    ) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .and_then(|d| f(d).as_deref())
    }

    pub fn get_freq_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.freq_input.as_str())
    }

    pub fn get_gain_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.gain_input.as_str())
    }

    pub fn get_q_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.q_input.as_str())
    }

    pub fn get_freq_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.freq_error)
    }

    pub fn get_gain_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.gain_error)
    }

    pub fn get_q_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.q_error)
    }

    pub fn has_errors(&self) -> bool {
        self.active_draft.as_ref().is_some_and(|d| d.has_errors())
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
            gain_input: format!("{:.2}", filter.gain), // Format to 2 decimal places
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolsTab {
    #[default]
    Preset,
    AutoEq,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfirmAction {
    #[default]
    None,
    ResetFilters,
    DeleteProfile,
    ImportAutoEQ {
        data: crate::models::PEQData,
        default_name: String,
    },
    OverwriteProfile {
        name: String,
        data: crate::models::PEQData,
    },
    PullDevice,
    ExitWithUnsavedChanges(iced::window::Id),
}

#[derive(Debug, Clone, Default)]
pub struct EditorData {
    pub filters: Vec<Filter>,
    pub global_gain: i8,
}

#[derive(Debug, Clone, Default)]
pub struct EditorSession {
    pub is_dirty: bool,
    pub is_autoeq_active: bool,
    pub input_buffer: InputBuffer,
    pub pending_confirm: ConfirmAction,
    pub undo_stack: Vec<crate::models::PEQData>,
    pub redo_stack: Vec<crate::models::PEQData>,
    pub status_message: Option<StatusMessage>,
    pub import_name_input: String,
    pub new_profile_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct EditorUI {
    pub profiles: Vec<Profile>,
    pub selected_profile_name: Option<String>,
    pub profiles_dir_mtime: Option<std::time::SystemTime>,
    pub profile_search: String,
    pub diagnostics_errors_only: bool,
    pub show_diagnostics: bool,
    pub snap_to_iso_enabled: bool,
    pub active_tools_tab: ToolsTab,
}

#[derive(Debug, Clone, Default)]
pub struct EditorState {
    pub data: EditorData,
    pub session: EditorSession,
    pub ui: EditorUI,
}

pub const MAX_UNDO: usize = 50;

impl EditorState {
    pub fn push_undo(&mut self) {
        let snapshot = crate::models::PEQData {
            filters: self.data.filters.clone(),
            global_gain: self.data.global_gain,
        };
        self.session.undo_stack.push(snapshot);
        if self.session.undo_stack.len() > MAX_UNDO {
            self.session.undo_stack.remove(0);
        }
        self.session.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PEQData;

    #[test]
    fn push_undo_clears_redo_and_adds_undo_entry() {
        let mut state = EditorState::default();
        state.session.redo_stack.push(PEQData {
            filters: vec![],
            global_gain: 0,
        });
        assert_eq!(state.session.undo_stack.len(), 0);
        assert_eq!(state.session.redo_stack.len(), 1);

        state.push_undo();

        assert_eq!(state.session.undo_stack.len(), 1);
        assert_eq!(state.session.redo_stack.len(), 0);
    }

    #[test]
    fn push_undo_trims_to_max() {
        let mut state = EditorState::default();
        for _ in 0..MAX_UNDO + 5 {
            state.push_undo();
        }
        assert_eq!(state.session.undo_stack.len(), MAX_UNDO);
    }
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
    pub connection_generation: u64,
    pub suspend_status_polling: bool,
    pub last_auto_reconnect_attempt: Option<std::time::Instant>,
    pub auto_reconnect_attempts: u32,
    pub last_profile_check: Option<std::time::Instant>,
}
