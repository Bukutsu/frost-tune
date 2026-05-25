// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

pub mod autoeq;
pub mod connection;
pub mod editor;
pub mod hardware;
pub mod profiles;

use crate::ui::messages::{
    AutoEqMessage, ConnectionMessage, DiagnosticsMessage, EditorMessage, Message, ProfilesMessage,
};
use crate::ui::state::AppState;
use iced::Task;

use self::autoeq::handle_autoeq;
use self::connection::handle_connection;
use self::editor::handle_editor;
use self::hardware::handle_hardware;
use self::profiles::handle_profiles;

pub fn update(window: &mut AppState, message: Message) -> Task<Message> {
    let before_generation = window.editor.data.generation;
    let task = match message {
        Message::NoOp => Task::none(),
        // handle_connection
        Message::ClearStatusMessage(_)
        | Message::DismissConfirmDialog
        | Message::Connection(ConnectionMessage::ConnectPressed(_))
        | Message::Connection(ConnectionMessage::DisconnectPressed)
        | Message::Connection(ConnectionMessage::WorkerConnected(..))
        | Message::Connection(ConnectionMessage::WorkerDisconnected(..))
        | Message::Connection(ConnectionMessage::WorkerStatus(..))
        | Message::Tick(..)
        | Message::Profiles(ProfilesMessage::ProfilesDirMtimeChecked(_))
        | Message::Connection(ConnectionMessage::WorkerBackendReset)
        | Message::CloseRequested(_)
        | Message::ConfirmExit(_)
        | Message::SaveAndExit(_) => handle_connection(window, message),

        // handle_hardware
        Message::Editor(EditorMessage::PullPressed)
        | Message::Editor(EditorMessage::ConfirmPullPressed)
        | Message::Editor(EditorMessage::PushPressed)
        | Message::Editor(EditorMessage::ConfirmPushPressed)
        | Message::Editor(EditorMessage::WorkerPulled(..))
        | Message::Editor(EditorMessage::WorkerPushed(..)) => handle_hardware(window, message),

        // handle_editor
        Message::Editor(EditorMessage::BandGainChanged(..))
        | Message::Editor(EditorMessage::BandFreqChanged(..))
        | Message::Editor(EditorMessage::BandQChanged(..))
        | Message::Editor(EditorMessage::BandTypeChanged(..))
        | Message::Editor(EditorMessage::BandEnabledToggled(..))
        | Message::Editor(EditorMessage::BandGainInput(..))
        | Message::Editor(EditorMessage::BandFreqInput(..))
        | Message::Editor(EditorMessage::BandQInput(..))
        | Message::Editor(EditorMessage::BandFreqSliderChanged(..))
        | Message::Editor(EditorMessage::BandFreqSliderReleased(..))
        | Message::Editor(EditorMessage::BandGainReleased(..))
        | Message::Editor(EditorMessage::BandFreqInputCommit(..))
        | Message::Editor(EditorMessage::BandGainInputCommit(..))
        | Message::Editor(EditorMessage::BandQInputCommit(..))
        | Message::Editor(EditorMessage::BandFreqInputCancel(..))
        | Message::Editor(EditorMessage::BandGainInputCancel(..))
        | Message::Editor(EditorMessage::BandQInputCancel(..))
        | Message::Editor(EditorMessage::GlobalGainChanged(..))
        | Message::Editor(EditorMessage::ResetFiltersPressed)
        | Message::Editor(EditorMessage::ConfirmResetFilters)
        | Message::Diagnostics(DiagnosticsMessage::ToggleDiagnostics)
        | Message::Editor(EditorMessage::Undo)
        | Message::Editor(EditorMessage::Redo)
        | Message::Editor(EditorMessage::ToggleSnapToIso(_))
        | Message::Editor(EditorMessage::ToggleAutoPullOnConnect(..))
        | Message::Editor(EditorMessage::SettingsSaved { .. }) => handle_editor(window, message),

        // handle_autoeq
        Message::AutoEq(AutoEqMessage::ImportFromClipboard)
        | Message::AutoEq(AutoEqMessage::ImportClipboardReceived(..))
        | Message::AutoEq(AutoEqMessage::ImportClipboardFailed(..))
        | Message::AutoEq(AutoEqMessage::ExportAutoEQPressed)
        | Message::AutoEq(AutoEqMessage::ExportComplete)
        | Message::Diagnostics(DiagnosticsMessage::ToggleDiagnosticsErrorsOnly(..))
        | Message::Diagnostics(DiagnosticsMessage::CopyDiagnostics)
        | Message::Diagnostics(DiagnosticsMessage::ClearDiagnostics)
        | Message::Diagnostics(DiagnosticsMessage::ExportDiagnosticsToFile)
        | Message::Diagnostics(DiagnosticsMessage::DiagnosticsExported(..))
        | Message::Diagnostics(DiagnosticsMessage::DiagnosticsExportedToFile { .. }) => {
            handle_autoeq(window, message)
        }

        // handle_profiles
        Message::Profiles(ProfilesMessage::ReloadProfilesPressed)
        | Message::Profiles(ProfilesMessage::OpenProfilesDirPressed)
        | Message::Profiles(ProfilesMessage::ProfilesLoaded(..))
        | Message::Profiles(ProfilesMessage::ProfileSelected(..))
        | Message::Profiles(ProfilesMessage::ProfileNameInput(..))
        | Message::Profiles(ProfilesMessage::SaveProfilePressed)
        | Message::Profiles(ProfilesMessage::ConfirmDeleteProfile)
        | Message::Profiles(ProfilesMessage::ConfirmLoadProfile)
        | Message::Profiles(ProfilesMessage::DeleteProfilePressed)
        | Message::Profiles(ProfilesMessage::ImportFromFilePressed)
        | Message::Profiles(ProfilesMessage::ExportToFilePressed)
        | Message::Profiles(ProfilesMessage::FileImported(..))
        | Message::Profiles(ProfilesMessage::FileExported(..))
        | Message::AutoEq(AutoEqMessage::ImportNameInput(..))
        | Message::AutoEq(AutoEqMessage::ConfirmImportWithName)
        | Message::Profiles(ProfilesMessage::ConfirmOverwriteProfile)
        | Message::AutoEq(AutoEqMessage::ImportDirectlyToEditor)
        | Message::AutoEq(AutoEqMessage::ImportOverwriteActive)
        | Message::AutoEq(AutoEqMessage::ImportProfileSelected(..))
        | Message::AutoEq(AutoEqMessage::ImportTemporaryToggled(..))
        | Message::Profiles(ProfilesMessage::ProfileSearchInput(..))
        | Message::ToolsTabSelected(..)
        | Message::Profiles(ProfilesMessage::ProfileSaved { .. })
        | Message::Profiles(ProfilesMessage::ProfileDeleted { .. })
        | Message::Profiles(ProfilesMessage::ProfileImported { .. })
        | Message::Profiles(ProfilesMessage::ProfileExported { .. }) => {
            handle_profiles(window, message)
        }
        Message::Event(_) => iced::Task::none(),
        Message::CancelExit => {
            window.editor.session.pending_confirm =
                crate::ui::components::editor::ConfirmAction::None;
            iced::Task::none()
        }
        Message::SettingsLoaded(Ok(settings)) => {
            window.editor.ui.auto_pull_on_connect = settings.auto_pull_on_connect;
            iced::Task::none()
        }
        Message::SettingsLoaded(Err(e)) => {
            window
                .diagnostics
                .push(crate::diagnostics::DiagnosticEvent::new(
                    crate::diagnostics::LogLevel::Error,
                    crate::diagnostics::Source::UI,
                    format!("Failed to load settings: {}", e),
                ));
            iced::Task::none()
        }
    };
    if window.editor.data.generation != before_generation {
        let (combined, bands) = crate::ui::graph::EqGraph::compute_responses(
            &window.editor.data.filters,
            window.editor.data.global_gain,
        );
        window.editor.ui.graph_state.cached_combined_response = combined;
        window.editor.ui.graph_state.cached_band_responses = bands;
        window.editor.ui.graph_state.curve_cache.clear();
    }
    task
}
