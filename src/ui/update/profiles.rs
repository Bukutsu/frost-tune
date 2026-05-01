use crate::ui::state::{MainWindow, ConfirmAction};
use crate::ui::messages::{Message, StatusSeverity};
use crate::models::PEQData;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use iced::Task;

pub fn handle_profiles(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::ReloadProfilesPressed => {
            Task::perform(
                async move { crate::storage::load_all_profiles() },
                Message::ProfilesLoaded,
            )
        }
        Message::ProfilesLoaded(result) => {
            window.editor_state.profiles_dir_mtime = crate::storage::get_profiles_dir_mtime();
            match result {
                Ok((profiles, errors)) => {
                    let prev_count = window.editor_state.profiles.len();
                    window.editor_state.profiles = profiles;
                    
                    for err in &errors {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            err.clone(),
                        ));
                    }

                    if !errors.is_empty() {
                        window.set_status(
                            format!("Loaded {} profiles ({} failed to parse)", window.editor_state.profiles.len(), errors.len()),
                            StatusSeverity::Warning
                        )
                    } else if window.editor_state.profiles.len() != prev_count {
                         window.set_status(
                            format!("Profiles updated ({} total)", window.editor_state.profiles.len()),
                            StatusSeverity::Info
                        )
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::UI,
                        format!("Failed to load profiles: {}", e),
                    ));
                    window.set_status(format!("Failed to load profiles: {}", e), StatusSeverity::Error)
                }
            }
        }
        Message::ProfileSelected(name) => {
            if let Some(profile) = window.editor_state.profiles.iter().find(|p| p.name == name) {
                window.editor_state.filters = profile
                    .data
                    .filters
                    .clone()
                    .into_iter()
                    .map(|mut f| {
                        f.enabled = true;
                        f
                    })
                    .collect();
                window.editor_state.global_gain = profile.data.global_gain;
                window.editor_state.selected_profile_name = Some(name);
                window.editor_state.new_profile_name = profile.name.clone();
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    format!("Loaded profile: {}", profile.name),
                ));
                window.set_status(
                    format!("Loaded profile: {}", profile.name),
                    StatusSeverity::Info,
                )
            } else {
                Task::none()
            }
        }
        Message::ProfileNameInput(name) => {
            window.editor_state.new_profile_name = name;
            Task::none()
        }
        Message::SaveProfilePressed => {
            let name = window.editor_state.new_profile_name.trim().to_string();
            if name.is_empty() {
                return window.set_status("Invalid profile name", StatusSeverity::Warning);
            }
            let data = PEQData {
                filters: window.editor_state.filters.clone(),
                global_gain: window.editor_state.global_gain,
            };
            match crate::storage::save_profile(&name, &data) {
                Ok(_) => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Saved profile: {}", name),
                    ));
                    let reload_task = Task::perform(
                        async move { crate::storage::load_all_profiles() },
                        Message::ProfilesLoaded,
                    );
                    let status_task = window.set_status(
                        format!("Saved profile: {}", name),
                        StatusSeverity::Success,
                    );
                    Task::batch(vec![reload_task, status_task])
                }
                Err(e) => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Error,
                        Source::UI,
                        format!("Save failed: {}", e),
                    ));
                    window.set_status(format!("Failed to save: {}", e), StatusSeverity::Error)
                }
            }
        }
        Message::DeleteProfilePressed => {
            window.editor_state.pending_confirm = ConfirmAction::DeleteProfile;
            Task::none()
        }
        Message::ConfirmDeleteProfile => {
            if matches!(
                window.editor_state.pending_confirm,
                ConfirmAction::DeleteProfile
            ) {
                let name = match &window.editor_state.selected_profile_name {
                    Some(n) => n.clone(),
                    None => return Task::none(),
                };
                match crate::storage::delete_profile(&name) {
                    Ok(_) => {
                        window.editor_state.selected_profile_name = None;
                        window.editor_state.new_profile_name = String::new();
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Deleted profile: {}", name),
                        ));
                        let reload_task = Task::perform(
                            async move { crate::storage::load_all_profiles() },
                            Message::ProfilesLoaded,
                        );
                        let status_task = window.set_status(
                            format!("Deleted profile: {}", name),
                            StatusSeverity::Success,
                        );
                        window.editor_state.pending_confirm = ConfirmAction::None;
                        Task::batch(vec![reload_task, status_task])
                    }
                    Err(e) => {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Delete failed: {}", e),
                        ));
                        window.set_status(format!("Failed to delete: {}", e), StatusSeverity::Error)
                    }
                }
            } else {
                Task::none()
            }
        }
        Message::ImportFromFilePressed => {
            Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Frost-Tune Profile", &["json", "txt"])
                        .pick_file()
                        .await
                },
                |handle| Message::FileImported(handle.map(|h| h.path().to_path_buf())),
            )
        }
        Message::FileImported(path_opt) => {
            if let Some(path) = path_opt {
                match crate::storage::import_profile(&path) {
                    Ok(profile) => {
                        window.editor_state.profiles.push(profile.clone());
                        window.editor_state.selected_profile_name = Some(profile.name.clone());
                        window.editor_state.new_profile_name = profile.name.clone();
                        window.editor_state.filters = profile.data.filters.clone();
                        window.editor_state.global_gain = profile.data.global_gain;
                        window.set_status(
                            format!("Imported profile: {}", profile.name),
                            StatusSeverity::Success,
                        )
                    }
                    Err(e) => window.set_status(format!("Import failed: {}", e), StatusSeverity::Error),
                }
            } else {
                Task::none()
            }
        }
        Message::ExportToFilePressed => {
            let peq = PEQData {
                filters: window.editor_state.filters.clone(),
                global_gain: window.editor_state.global_gain,
            };
            let name = if window.editor_state.new_profile_name.is_empty() {
                "profile".to_string()
            } else {
                window.editor_state.new_profile_name.clone()
            };

            Task::perform(
                async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Frost-Tune Profile", &["json", "txt"])
                        .set_file_name(&format!("{}.txt", name))
                        .save_file()
                        .await
                },
                move |handle| Message::FileExported(handle.map(|h| h.path().to_path_buf()), peq),
            )
        }
        Message::FileExported(path_opt, peq) => {
            if let Some(path) = path_opt {
                match crate::storage::export_profile(&path, &peq) {
                    Ok(_) => window.set_status("Profile exported", StatusSeverity::Success),
                    Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
                }
            } else {
                Task::none()
            }
        }
        _ => Task::none(),
    }
}
