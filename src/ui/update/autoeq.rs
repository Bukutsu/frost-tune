use crate::ui::state::MainWindow;
use crate::ui::messages::{Message, StatusSeverity};
use crate::models::PEQData;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::autoeq;
use crate::ui::main_window::APP_VERSION;
use iced::{clipboard, Task};

pub fn handle_autoeq(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::ExportAutoEQPressed => {
            let peq = PEQData {
                filters: window.editor_state.filters.clone(),
                global_gain: window.editor_state.global_gain,
            };
            let output = autoeq::peq_to_autoeq(&peq);
            let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
            let status_task = window.set_status("Exported to clipboard", StatusSeverity::Success);
            Task::batch(vec![write_task, status_task])
        }
        Message::ExportComplete => Task::none(),
        Message::ImportFromClipboard => {
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::AutoEQ,
                "Import from clipboard started",
            ));
            clipboard::read().map(|result| match result {
                Some(text) => Message::ImportClipboardReceived(text),
                None => Message::ImportClipboardFailed("Clipboard empty or not text".into()),
            })
        }
        Message::ImportClipboardReceived(text) => match autoeq::parse_autoeq_text(&text) {
            Ok(peq) => {
                window.editor_state.pending_confirm = crate::ui::state::ConfirmAction::ImportAutoEQ(peq);
                Task::none()
            }
            Err(e) => {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::AutoEQ,
                    format!("Import failed: {}", e),
                ));
                window.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
            }
        },
        Message::ConfirmImportAutoEQ => {
            if let crate::ui::state::ConfirmAction::ImportAutoEQ(peq) = window.editor_state.pending_confirm.clone() {
                let enabled_count = peq.filters.iter().filter(|f| f.enabled).count();
                window.editor_state.filters = peq
                    .filters
                    .into_iter()
                    .map(|mut f| {
                        f.enabled = true;
                        f
                    })
                    .collect();
                window.editor_state.global_gain = peq.global_gain;
                window.editor_state.pending_confirm = crate::ui::state::ConfirmAction::None;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::AutoEQ,
                    format!("Import successful: {} filters", enabled_count),
                ));
                window.set_status(
                    format!("Imported {} filters", enabled_count),
                    StatusSeverity::Success,
                )
            } else {
                Task::none()
            }
        }
        Message::ImportClipboardFailed(msg) => {
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Error,
                Source::AutoEQ,
                msg.clone(),
            ));
            window.set_status(msg, StatusSeverity::Error)
        }
        Message::CopyDiagnostics => {
            let conn_str = format!("{:?}", window.connection_status);
            let output = crate::diagnostics::format_diagnostics(
                &window.diagnostics,
                APP_VERSION,
                &conn_str,
            );
            let write_task = clipboard::write(output).map(|()| Message::ExportComplete);
            let status_task = window.set_status("Diagnostics copied", StatusSeverity::Info);
            Task::batch(vec![write_task, status_task])
        }
        Message::ClearDiagnostics => {
            window.diagnostics.clear();
            window.set_status("Diagnostics cleared", StatusSeverity::Info)
        }
        Message::ToggleDiagnosticsErrorsOnly(v) => {
            window.editor_state.diagnostics_errors_only = v;
            Task::none()
        }
        Message::ExportDiagnosticsToFile => {
            let conn_str = format!("{:?}", window.connection_status);
            let output = crate::diagnostics::format_diagnostics(
                &window.diagnostics,
                APP_VERSION,
                &conn_str,
            );
            let now = chrono::Local::now();
            let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
            let path = dirs::document_dir()
                .or_else(dirs::data_dir)
                .unwrap_or_else(std::env::temp_dir)
                .join(&filename);
            match std::fs::write(&path, output) {
                Ok(_) => Task::done(Message::DiagnosticsExported(path.display().to_string())),
                Err(e) => {
                    window.set_status(format!("Export failed: {}", e), StatusSeverity::Error)
                }
            }
        }
        Message::DiagnosticsExported(name) => {
            window.set_status(format!("Saved to {}", name), StatusSeverity::Success)
        }
        _ => Task::none(),
    }
}
