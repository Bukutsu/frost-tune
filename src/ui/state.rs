use crate::diagnostics::DiagnosticsStore;
use crate::hardware::worker::UsbWorker;
use crate::models::Filter;
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
pub struct EditorState {
    pub filters: Vec<Filter>,
    pub global_gain: i8,
    pub status_message: Option<StatusMessage>,
    pub diagnostics_errors_only: bool,
    pub profiles: Vec<Profile>,
    pub selected_profile_name: Option<String>,
    pub new_profile_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct OperationLock {
    pub is_pulling: bool,
    pub is_pushing: bool,
    pub is_connecting: bool,
}

#[derive(Default)]
pub struct MainWindow {
    pub connection_status: ConnectionStatus,
    pub disconnect_reason: DisconnectReason,
    pub editor_state: EditorState,
    pub operation_lock: OperationLock,
    pub worker: Option<Arc<UsbWorker>>,
    pub diagnostics: DiagnosticsStore,
}
