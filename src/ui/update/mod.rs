pub mod connection;
pub mod hardware;
pub mod editor;
pub mod autoeq;
pub mod profiles;

use crate::ui::state::MainWindow;
use crate::ui::messages::Message;
use iced::Task;

use self::connection::handle_connection;
use self::hardware::handle_hardware;
use self::editor::handle_editor;
use self::autoeq::handle_autoeq;
use self::profiles::handle_profiles;

pub fn update(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        // handle_connection
        Message::ClearStatusMessage |
        Message::DismissConfirmDialog |
        Message::DeviceSelected(_) |
        Message::ConnectPressed(_) |
        Message::ConfirmElevatedConnect(_) |
        Message::DisconnectPressed |
        Message::WorkerConnected(..) |
        Message::WorkerDisconnected(..) |
        Message::WorkerStatus(..) |
        Message::Tick(..) => handle_connection(window, message),

        // handle_hardware
        Message::PullPressed |
        Message::PushPressed |
        Message::WorkerPulled(..) |
        Message::WorkerPushed(..) => handle_hardware(window, message),

        // handle_editor
        Message::BandGainChanged(..) |
        Message::BandFreqChanged(..) |
        Message::BandQChanged(..) |
        Message::BandTypeChanged(..) |
        Message::BandGainInput(..) |
        Message::BandFreqInput(..) |
        Message::BandQInput(..) |
        Message::BandFreqSliderChanged(..) |
        Message::BandFreqInputCommit(..) |
        Message::BandGainInputCommit(..) |
        Message::BandQInputCommit(..) |
        Message::BandFreqInputCancel(..) |
        Message::BandGainInputCancel(..) |
        Message::BandQInputCancel(..) |
        Message::GlobalGainChanged(..) |
        Message::ResetFiltersPressed |
        Message::ConfirmResetFilters => handle_editor(window, message),

        // handle_autoeq
        Message::ImportFromClipboard |
        Message::ImportClipboardReceived(..) |
        Message::ImportClipboardFailed(..) |
        Message::ExportAutoEQPressed |
        Message::ExportComplete |
        Message::ToggleDiagnosticsErrorsOnly(..) |
        Message::CopyDiagnostics |
        Message::ClearDiagnostics |
        Message::ExportDiagnosticsToFile |
        Message::DiagnosticsExported(..) => handle_autoeq(window, message),

        // handle_profiles
        Message::ProfilesLoaded(..) |
        Message::ProfileSelected(..) |
        Message::ProfileNameInput(..) |
        Message::SaveProfilePressed |
        Message::ConfirmDeleteProfile |
        Message::DeleteProfilePressed |
        Message::ImportFromFilePressed |
        Message::ExportToFilePressed |
        Message::FileImported(..) |
        Message::FileExported(..) => handle_profiles(window, message),
    }
}
