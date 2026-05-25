// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

pub use crate::ui::components::autoeq::AutoEqMessage;
pub use crate::ui::components::connection::ConnectionMessage;
pub use crate::ui::components::diagnostics::DiagnosticsMessage;
pub use crate::ui::components::editor::EditorMessage;
pub use crate::ui::components::profiles::ProfilesMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum StatusSeverity {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub id: usize,
    pub content: String,
    pub severity: StatusSeverity,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum SaveContext {
    Standard,
    Exit(iced::window::Id),
    LoadProfile(String),
    ImportOverwrite,
    ImportWithName,
    Overwrite,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolsTab {
    #[default]
    Preset,
    AutoEq,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EqSource {
    #[default]
    Default,
    Pulled,
    Imported,
    Profile,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,

    Connection(ConnectionMessage),
    Editor(EditorMessage),
    Profiles(ProfilesMessage),
    AutoEq(AutoEqMessage),
    Diagnostics(DiagnosticsMessage),

    Tick(std::time::Instant),
    Event(iced::Event),
    CloseRequested(iced::window::Id),
    ConfirmExit(iced::window::Id),
    CancelExit,
    SettingsLoaded(Result<crate::storage::Settings, crate::error::AppError>),
    ToolsTabSelected(ToolsTab),
    ClearStatusMessage(usize),
    DismissConfirmDialog,
    SaveAndExit(iced::window::Id),
}
