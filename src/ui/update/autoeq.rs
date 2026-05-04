use crate::autoeq;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::models::PEQData;
use crate::ui::main_window::APP_VERSION;
use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::MainWindow;
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
                window.editor_state.pending_confirm =
                    crate::ui::state::ConfirmAction::ImportAutoEQ(peq);
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
            if let crate::ui::state::ConfirmAction::ImportAutoEQ(peq) =
                window.editor_state.pending_confirm.clone()
            {
                let num_bands = window.num_bands();
                let freq_range = window.freq_range();
                let gain_range = window.gain_range();
                let q_range = window.q_range();

                let mut filters = peq.filters;
                let was_truncated = filters.len() > num_bands;
                if was_truncated {
                    filters.truncate(num_bands);
                }

                let enabled_count = filters.iter().filter(|f| f.enabled).count();
                window.editor_state.filters = filters
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut f)| {
                        f.index = i as u8;
                        f.enabled = true;
                        f.clamp(freq_range, gain_range, q_range);
                        f
                    })
                    .collect();

                // Pad if needed
                while window.editor_state.filters.len() < num_bands {
                    window
                        .editor_state
                        .filters
                        .push(crate::models::Filter::enabled(
                            window.editor_state.filters.len() as u8,
                            false,
                        ));
                }

                window.editor_state.global_gain = peq.global_gain;
                window.editor_state.is_autoeq_active = true;
                window.editor_state.pending_confirm = crate::ui::state::ConfirmAction::None;

                if was_truncated {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::AutoEQ,
                        format!("Import truncated to {} bands", num_bands),
                    ));
                    window.set_status(
                        format!(
                            "Imported {} filters (truncated to {})",
                            enabled_count, num_bands
                        ),
                        StatusSeverity::Warning,
                    )
                } else {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::AutoEQ,
                        format!("Import successful: {} filters", enabled_count),
                    ));
                    window.set_status(
                        format!("Imported {} filters", enabled_count),
                        StatusSeverity::Success,
                    )
                }
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
            let output =
                crate::diagnostics::format_diagnostics(&window.diagnostics, APP_VERSION, &conn_str);
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
            let output =
                crate::diagnostics::format_diagnostics(&window.diagnostics, APP_VERSION, &conn_str);
            let now = chrono::Local::now();
            let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
            let path = dirs::document_dir()
                .or_else(dirs::data_dir)
                .unwrap_or_else(std::env::temp_dir)
                .join(&filename);
            match std::fs::write(&path, output) {
                Ok(_) => Task::done(Message::DiagnosticsExported(path.display().to_string())),
                Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
            }
        }
        Message::DiagnosticsExported(name) => {
            window.set_status(format!("Saved to {}", name), StatusSeverity::Success)
        }
        _ => Task::none(),
    }
}
