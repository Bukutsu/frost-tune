// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::autoeq;
use crate::core::PEQData;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::ui::main_window::APP_VERSION;
use crate::ui::messages::*;
use crate::ui::state::AppState;
use iced::{clipboard, Task};

pub fn handle_autoeq(window: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::AutoEq(AutoEqMessage::ExportAutoEQPressed) => {
            let peq = PEQData {
                filters: window.editor.data.filters.clone(),
                global_gain: window.editor.data.global_gain,
            };
            let output = autoeq::peq_to_autoeq(&peq);
            let write_task =
                clipboard::write(output).map(|()| Message::AutoEq(AutoEqMessage::ExportComplete));
            let status_task = window.set_status("Exported to clipboard", StatusSeverity::Success);
            Task::batch(vec![write_task, status_task])
        }
        Message::AutoEq(AutoEqMessage::ExportComplete) => Task::none(),
        Message::AutoEq(AutoEqMessage::ImportFromClipboard) => {
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::AutoEQ,
                "Import from clipboard started",
            ));
            clipboard::read().map(|result| match result {
                Some(text) => Message::AutoEq(AutoEqMessage::ImportClipboardReceived(text)),
                None => Message::AutoEq(AutoEqMessage::ImportClipboardFailed(
                    "Clipboard empty or not text".into(),
                )),
            })
        }
        Message::AutoEq(AutoEqMessage::ImportClipboardReceived(text)) => {
            match autoeq::parse_autoeq_text(&text) {
                Ok((mut peq, warnings)) => {
                    if let Some(profile) = window.active_device() {
                        peq.clamp_to_capabilities(&profile.capabilities());
                    } else {
                        peq.clamp_to_capabilities(
                            &crate::core::device::capabilities::DESKTOP_DAC_CAPS,
                        );
                    }
                    if !warnings.is_empty() {
                        for w in &warnings {
                            window.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Warn,
                                Source::AutoEQ,
                                format!("Import warning: {}", w),
                            ));
                        }
                    }
                    window.editor.session.import_temporary = false;
                    window.editor.session.import_name_input.clear();
                    window.editor.session.pending_confirm =
                        crate::ui::components::editor::ConfirmAction::ImportAutoEQ {
                            data: std::sync::Arc::new(peq),
                            default_name: format!(
                                "Imported {}",
                                chrono::Local::now().format("%Y-%m-%d %H:%M")
                            ),
                        };
                    if !warnings.is_empty() {
                        window.set_status(
                            format!("Import parsed with warnings: {}", warnings.join("; ")),
                            StatusSeverity::Warning,
                        )
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::AutoEQ,
                        format!("Clipboard parse failed: {}", e),
                    ));
                    window.set_status("Clipboard doesn't contain an EQ", StatusSeverity::Warning)
                }
            }
        }
        Message::AutoEq(AutoEqMessage::ImportClipboardFailed(msg)) => {
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Warn,
                Source::AutoEQ,
                msg.clone(),
            ));
            window.set_status(msg, StatusSeverity::Warning)
        }
        Message::Diagnostics(DiagnosticsMessage::CopyDiagnostics) => {
            let conn_str = format!("{:?}", window.connection.status);
            let output =
                crate::diagnostics::format_diagnostics(&window.diagnostics, APP_VERSION, &conn_str);
            let write_task =
                clipboard::write(output).map(|()| Message::AutoEq(AutoEqMessage::ExportComplete));
            let status_task = window.set_status("Diagnostics copied", StatusSeverity::Info);
            Task::batch(vec![write_task, status_task])
        }
        Message::Diagnostics(DiagnosticsMessage::ClearDiagnostics) => {
            window.diagnostics.clear();
            window.set_status("Diagnostics cleared", StatusSeverity::Info)
        }
        Message::Diagnostics(DiagnosticsMessage::ToggleDiagnosticsErrorsOnly(v)) => {
            window.editor.ui.diagnostics_errors_only = v;
            Task::none()
        }
        Message::Diagnostics(DiagnosticsMessage::ExportDiagnosticsToFile) => {
            let conn_str = format!("{:?}", window.connection.status);
            let output =
                crate::diagnostics::format_diagnostics(&window.diagnostics, APP_VERSION, &conn_str);
            let now = chrono::Local::now();
            let filename = format!("frost_tune_diag_{}.txt", now.format("%Y%m%d_%H%M%S"));
            let path = dirs::document_dir()
                .or_else(dirs::data_dir)
                .unwrap_or_else(std::env::temp_dir)
                .join(&filename);
            let path_str = path.display().to_string();

            Task::perform(
                async move {
                    tokio::fs::write(&path, output).await.map_err(|e| {
                        crate::error::AppError::new(
                            crate::error::ErrorKind::StorageError,
                            e.to_string(),
                        )
                    })
                },
                move |result| {
                    Message::Diagnostics(DiagnosticsMessage::DiagnosticsExportedToFile {
                        path: path_str.clone(),
                        result,
                    })
                },
            )
        }
        Message::Diagnostics(DiagnosticsMessage::DiagnosticsExportedToFile { path, result }) => {
            match result {
                Ok(_) => window.set_status(format!("Saved to {}", path), StatusSeverity::Success),
                Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
            }
        }
        _ => Task::none(),
    }
}
