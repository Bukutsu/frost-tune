// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

pub mod autoeq;
pub mod connection;
pub mod editor;
pub mod hardware;
pub mod profiles;

use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use iced::Task;

use self::autoeq::handle_autoeq;
use self::connection::handle_connection;
use self::editor::handle_editor;
use self::hardware::handle_hardware;
use self::profiles::handle_profiles;

pub fn update(window: &mut MainWindow, message: Message) -> Task<Message> {
    let before_generation = window.editor_state.data.generation;
    let task = match message {
        Message::None => Task::none(),
        // handle_connection
        Message::ClearStatusMessage(_)
        | Message::DismissConfirmDialog
        | Message::ConnectPressed(_)
        | Message::DisconnectPressed
        | Message::WorkerConnected(..)
        | Message::WorkerDisconnected(..)
        | Message::WorkerStatus(..)
        | Message::Tick(..)
        | Message::ProfilesDirMtimeChecked(_)
        | Message::WorkerBackendReset
        | Message::WindowCloseRequested(_)
        | Message::ConfirmExit(_)
        | Message::SaveAndExit(_) => handle_connection(window, message),

        // handle_hardware
        Message::PullPressed
        | Message::ConfirmPullPressed
        | Message::PushPressed
        | Message::ConfirmPushPressed
        | Message::WorkerPulled(..)
        | Message::WorkerPushed(..) => handle_hardware(window, message),

        // handle_editor
        Message::BandGainChanged(..)
        | Message::BandFreqChanged(..)
        | Message::BandQChanged(..)
        | Message::BandTypeChanged(..)
        | Message::BandEnabledToggled(..)
        | Message::BandGainInput(..)
        | Message::BandFreqInput(..)
        | Message::BandQInput(..)
        | Message::BandFreqSliderChanged(..)
        | Message::BandFreqSliderReleased(..)
        | Message::BandGainReleased(..)
        | Message::BandFreqInputCommit(..)
        | Message::BandGainInputCommit(..)
        | Message::BandQInputCommit(..)
        | Message::BandFreqInputCancel(..)
        | Message::BandGainInputCancel(..)
        | Message::BandQInputCancel(..)
        | Message::GlobalGainChanged(..)
        | Message::ResetFiltersPressed
        | Message::ConfirmResetFilters
        | Message::ToggleDiagnostics
        | Message::Undo
        | Message::Redo
        | Message::ToggleSnapToIso(_)
        | Message::ToggleAutoPullOnConnect(..) => handle_editor(window, message),

        // handle_autoeq
        Message::ImportFromClipboard
        | Message::ImportClipboardReceived(..)
        | Message::ImportClipboardFailed(..)
        | Message::ExportAutoEQPressed
        | Message::ExportComplete
        | Message::ToggleDiagnosticsErrorsOnly(..)
        | Message::CopyDiagnostics
        | Message::ClearDiagnostics
        | Message::ExportDiagnosticsToFile
        | Message::DiagnosticsExported(..) => handle_autoeq(window, message),

        // handle_profiles
        Message::ReloadProfilesPressed
        | Message::OpenProfilesDirPressed
        | Message::ProfilesLoaded(..)
        | Message::ProfileSelected(..)
        | Message::ProfileNameInput(..)
        | Message::SaveProfilePressed
        | Message::ConfirmDeleteProfile
        | Message::ConfirmLoadProfile
        | Message::DeleteProfilePressed
        | Message::ImportFromFilePressed
        | Message::ExportToFilePressed
        | Message::FileImported(..)
        | Message::FileExported(..)
        | Message::ImportNameInput(..)
        | Message::ConfirmImportWithName
        | Message::ConfirmOverwriteProfile
        | Message::ImportDirectlyToEditor
        | Message::ImportOverwriteActive
        | Message::ImportProfileSelected(..)
        | Message::ProfileSearchInput(..)
        | Message::ToolsTabSelected(..) => handle_profiles(window, message),
    };
    if window.editor_state.data.generation != before_generation {
        let (combined, bands) = crate::ui::graph::EqGraph::compute_responses(
            &window.editor_state.data.filters,
            window.editor_state.data.global_gain,
        );
        window.editor_state.ui.graph_state.cached_combined_response = combined;
        window.editor_state.ui.graph_state.cached_band_responses = bands;
        window.editor_state.ui.graph_state.curve_cache.clear();
    }
    task
}
