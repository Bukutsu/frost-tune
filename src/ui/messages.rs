// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::AppError;
use crate::hardware::worker::WorkerStatus;
use crate::models::{ConnectionResult, DeviceInfo, OperationResult};
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
    ConnectPressed(DeviceInfo),
    DisconnectPressed,
    PullPressed,
    ConfirmPullPressed,
    PushPressed,
    ConfirmPushPressed,
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
    BandEnabledToggled(usize, bool),
    BandGainInput(usize, String),
    BandFreqInput(usize, String),
    BandQInput(usize, String),
    BandFreqSliderChanged(usize, f64),
    BandFreqSliderReleased(usize),
    BandGainReleased(usize),
    BandFreqInputCommit(usize),
    BandGainInputCommit(usize),
    BandQInputCommit(usize),
    BandFreqInputCancel(usize),
    BandGainInputCancel(usize),
    BandQInputCancel(usize),
    GlobalGainChanged(i8),
    ResetFiltersPressed,
    ConfirmResetFilters,
    ImportNameInput(String),
    ConfirmImportWithName,
    ConfirmOverwriteProfile,
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
    ProfilesLoaded(Result<(Vec<Profile>, Vec<String>), AppError>),
    ProfilesDirMtimeChecked(Option<std::time::SystemTime>),
    WorkerBackendReset,
    ReloadProfilesPressed,
    OpenProfilesDirPressed,

    ProfileSelected(String),
    ProfileNameInput(String),
    SaveProfilePressed,
    ConfirmDeleteProfile,
    ConfirmLoadProfile,
    DeleteProfilePressed,
    ImportFromFilePressed,
    ExportToFilePressed,
    FileImported(Option<std::path::PathBuf>),
    FileExported(Option<std::path::PathBuf>, crate::models::PEQData),

    ClearStatusMessage(u64),
    DismissConfirmDialog,
    WindowCloseRequested(iced::window::Id),
    ConfirmExit(iced::window::Id),
    SaveAndExit(iced::window::Id),

    Undo,
    Redo,

    ToggleSnapToIso(bool),
    ProfileSearchInput(String),
    ToolsTabSelected(crate::ui::state::ToolsTab),
}
