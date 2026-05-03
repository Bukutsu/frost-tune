use crate::hardware::worker::WorkerStatus;
use crate::models::{DeviceInfo, ConnectionResult, OperationResult};
use crate::storage::Profile;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub id: u64,
    pub content: String,
    pub severity: StatusSeverity,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    DeviceSelected(usize),
    ConnectPressed(DeviceInfo),
    ConfirmElevatedConnect(DeviceInfo),
    DisconnectPressed,
    PullPressed,
    ConfirmPullPressed,
    PushPressed,
    WorkerConnected(ConnectionResult),
    WorkerDisconnected(OperationResult),
    WorkerPulled(OperationResult),
    WorkerPushed(OperationResult),
    WorkerStatus(WorkerStatus),
    Tick(std::time::Instant),
    BandGainChanged(usize, f64),
    BandFreqChanged(usize, u16),
    BandQChanged(usize, f64),
    BandTypeChanged(usize, crate::models::FilterType),
    BandGainInput(usize, String),
    BandFreqInput(usize, String),
    BandQInput(usize, String),
    BandFreqSliderChanged(usize, f64),
    BandFreqInputCommit(usize),
    BandGainInputCommit(usize),
    BandQInputCommit(usize),
    BandFreqInputCancel(usize),
    BandGainInputCancel(usize),
    BandQInputCancel(usize),
    GlobalGainChanged(i8),
    ResetFiltersPressed,
    ConfirmResetFilters,
    ConfirmImportAutoEQ,
    ImportFromClipboard,
    ImportClipboardReceived(String),
    ImportClipboardFailed(String),
    ExportAutoEQPressed,
    ExportComplete,
    CopyDiagnostics,
    ClearDiagnostics,
    ToggleDiagnostics,
    ToggleDiagnosticsErrorsOnly(bool),
    ExportDiagnosticsToFile,
    DiagnosticsExported(String),
    ProfilesLoaded(Result<(Vec<Profile>, Vec<String>), String>),
    ProfilesDirMtimeChecked(Option<std::time::SystemTime>),
    WorkerBackendReset,
    ReloadProfilesPressed,
    OpenProfilesDirPressed,

    ProfileSelected(String),
    ProfileNameInput(String),
    SaveProfilePressed,
    ConfirmDeleteProfile,
    DeleteProfilePressed,
    ImportFromFilePressed,
    ExportToFilePressed,
    FileImported(Option<std::path::PathBuf>),
    FileExported(Option<std::path::PathBuf>, crate::models::PEQData),

    ClearStatusMessage(u64),
    DismissConfirmDialog,
}
