// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::models::PEQData;
use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::{ConfirmAction, MainWindow};
use iced::Task;

fn reload_profiles_task() -> Task<Message> {
    Task::perform(
        async move { crate::storage::load_all_profiles() },
        Message::ProfilesLoaded,
    )
}

fn apply_peq_to_editor(window: &mut MainWindow, peq: PEQData) -> (bool, usize) {
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
    window.editor_state.data.filters = filters
        .into_iter()
        .enumerate()
        .map(|(i, mut f)| {
            f.index = i as u8;
            f.enabled = true;
            f.clamp(freq_range, gain_range, q_range);
            f
        })
        .collect();

    while window.editor_state.data.filters.len() < num_bands {
        window
            .editor_state
            .data
            .filters
            .push(crate::models::Filter::enabled(
                window.editor_state.data.filters.len() as u8,
                false,
            ));
    }

    window.editor_state.data.global_gain = peq.global_gain;
    window.editor_state.session.is_autoeq_active = true;

    (was_truncated, enabled_count)
}

fn check_overwrite_and_save(
    window: &mut MainWindow,
    name: String,
    data: PEQData,
    reload_on_save: bool,
) -> Task<Message> {
    let name_exists = window
        .editor_state
        .ui
        .profiles
        .iter()
        .any(|p| p.name == name);

    if name_exists {
        window.editor_state.session.pending_confirm =
            ConfirmAction::OverwriteProfile { name, data };
        return Task::none();
    }

    do_save_profile(window, name, data, reload_on_save)
}

fn do_save_profile(
    window: &mut MainWindow,
    name: String,
    data: PEQData,
    reload_on_save: bool,
) -> Task<Message> {
    match crate::storage::save_profile(&name, &data) {
        Ok(_) => {
            window.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Saved profile: {}", name),
            ));
            window.editor_state.session.new_profile_name = name.clone();
            window.editor_state.ui.selected_profile_name = Some(name.clone());
            window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;
            let mut tasks = Vec::new();
            if reload_on_save {
                tasks.push(reload_profiles_task());
            }
            tasks.push(
                window.set_status(format!("Saved profile: {}", name), StatusSeverity::Success),
            );
            Task::batch(tasks)
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

pub fn handle_profiles(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::ReloadProfilesPressed => reload_profiles_task(),
        Message::OpenProfilesDirPressed => {
            if let Err(e) = crate::storage::open_profiles_dir() {
                window.set_status(
                    format!("Failed to open folder: {}", e),
                    StatusSeverity::Error,
                )
            } else {
                Task::none()
            }
        }
        Message::ProfilesLoaded(result) => {
            window.editor_state.ui.profiles_dir_mtime = crate::storage::get_profiles_dir_mtime();
            match result {
                Ok((profiles, errors)) => {
                    let prev_count = window.editor_state.ui.profiles.len();
                    window.editor_state.ui.profiles = profiles;

                    for err in &errors {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            err.clone(),
                        ));
                    }

                    if !errors.is_empty() {
                        window.set_status(
                            format!(
                                "Loaded {} profiles ({} failed to parse)",
                                window.editor_state.ui.profiles.len(),
                                errors.len()
                            ),
                            StatusSeverity::Warning,
                        )
                    } else if window.editor_state.ui.profiles.len() != prev_count {
                        window.set_status(
                            format!(
                                "Profiles updated ({} total)",
                                window.editor_state.ui.profiles.len()
                            ),
                            StatusSeverity::Info,
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
                    window.set_status(
                        format!("Failed to load profiles: {}", e),
                        StatusSeverity::Error,
                    )
                }
            }
        }
        Message::ProfileSelected(name) => {
            if window.editor_state.session.is_dirty {
                window.editor_state.session.pending_confirm =
                    crate::ui::state::ConfirmAction::LoadProfile { name };
                return Task::none();
            }

            window.editor_state.push_undo();
            let (profile_name, was_truncated) = match window
                .editor_state
                .ui
                .profiles
                .iter()
                .find(|p| p.name == name)
            {
                Some(profile) => {
                    let profile_name = profile.name.clone();
                    let (was_truncated, _) = apply_peq_to_editor(window, profile.data.clone());
                    window.editor_state.ui.selected_profile_name = Some(name);
                    window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;
                    window.editor_state.session.new_profile_name = profile_name.clone();
                    window.editor_state.ui.profile_search.clear();
                    window.editor_state.session.is_autoeq_active = false;
                    (profile_name, was_truncated)
                }
                None => return Task::none(),
            };

            if was_truncated {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Warn,
                    Source::UI,
                    format!(
                        "Profile {} truncated to {} bands",
                        profile_name,
                        window.num_bands()
                    ),
                ));
                window.set_status(
                    format!(
                        "Loaded profile: {} (truncated to {})",
                        profile_name,
                        window.num_bands()
                    ),
                    StatusSeverity::Warning,
                )
            } else {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    format!("Loaded profile: {}", profile_name),
                ));
                window.set_status(
                    format!("Loaded profile: {}", profile_name),
                    StatusSeverity::Info,
                )
            }
        }
        Message::ProfileNameInput(name) => {
            window.editor_state.session.new_profile_name = name;
            Task::none()
        }
        Message::ImportNameInput(name) => {
            window.editor_state.session.import_name_input = name;
            Task::none()
        }
        Message::SaveProfilePressed => {
            let name = window
                .editor_state
                .session
                .new_profile_name
                .trim()
                .to_string();
            if name.is_empty() {
                return window.set_status("Invalid profile name", StatusSeverity::Warning);
            }
            let data = PEQData {
                filters: window.editor_state.data.filters.clone(),
                global_gain: window.editor_state.data.global_gain,
            };
            check_overwrite_and_save(window, name, data, true)
        }
        Message::ConfirmImportWithName => {
            if let ConfirmAction::ImportAutoEQ { data, .. } =
                window.editor_state.session.pending_confirm.clone()
            {
                let name = window
                    .editor_state
                    .session
                    .import_name_input
                    .trim()
                    .to_string();

                if name.is_empty() {
                    return window
                        .set_status("Profile name cannot be empty", StatusSeverity::Warning);
                }

                let name_exists = window
                    .editor_state
                    .ui
                    .profiles
                    .iter()
                    .any(|p| p.name == name);

                if name_exists {
                    window.editor_state.session.pending_confirm =
                        ConfirmAction::OverwriteProfile { name, data };
                    return Task::none();
                }

                match crate::storage::save_profile(&name, &data) {
                    Ok(_) => {
                        window.editor_state.push_undo();
                        let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                        window.editor_state.session.import_name_input = String::new();
                        window.editor_state.session.pending_confirm = ConfirmAction::None;
                        window.editor_state.ui.selected_profile_name = Some(name.clone());
                        window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;

                        let mut tasks = vec![reload_profiles_task()];

                        if was_truncated {
                            window.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Warn,
                                Source::UI,
                                format!("Import truncated to {} bands", window.num_bands()),
                            ));
                            tasks.push(window.set_status(
                                format!(
                                    "Imported {} filters (truncated to {})",
                                    enabled_count,
                                    window.num_bands()
                                ),
                                StatusSeverity::Warning,
                            ));
                        } else {
                            window.diagnostics.push(DiagnosticEvent::new(
                                LogLevel::Info,
                                Source::UI,
                                format!("Import successful: {} filters", enabled_count),
                            ));
                            tasks.push(window.set_status(
                                format!("Imported {} filters", enabled_count),
                                StatusSeverity::Success,
                            ));
                        }
                        Task::batch(tasks)
                    }
                    Err(e) => {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Error,
                            Source::UI,
                            format!("Import failed: {}", e),
                        ));
                        window.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
                    }
                }
            } else {
                Task::none()
            }
        }
        Message::ConfirmOverwriteProfile => {
            if let ConfirmAction::OverwriteProfile { name, data } =
                window.editor_state.session.pending_confirm.clone()
            {
                match crate::storage::save_profile(&name, &data) {
                    Ok(_) => {
                        window.editor_state.push_undo();
                        apply_peq_to_editor(window, data);
                        window.editor_state.session.pending_confirm = ConfirmAction::None;
                        window.editor_state.session.new_profile_name = name.clone();
                        window.editor_state.ui.selected_profile_name = Some(name.clone());
                        window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Overwritten profile: {}", name),
                        ));
                        let reload_task = reload_profiles_task();
                        let status_task = window.set_status(
                            format!("Overwritten profile: {}", name),
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
            } else {
                Task::none()
            }
        }
        Message::DeleteProfilePressed => {
            window.editor_state.session.pending_confirm = ConfirmAction::DeleteProfile;
            Task::none()
        }
        Message::ConfirmDeleteProfile => {
            if matches!(
                window.editor_state.session.pending_confirm,
                ConfirmAction::DeleteProfile
            ) {
                window.editor_state.session.pending_confirm = ConfirmAction::None;
            } else {
                return Task::none();
            }

            if let Some(name) = window.editor_state.ui.selected_profile_name.clone() {
                match crate::storage::delete_profile(&name) {
                    Ok(_) => {
                        window.editor_state.session.is_dirty = false;
                        window.editor_state.ui.selected_profile_name = None;
                        window.editor_state.ui.eq_source = crate::ui::state::EqSource::Default;
                        window.editor_state.session.new_profile_name.clear();
                        return window.set_status(
                            format!("Deleted profile: {}", name),
                            StatusSeverity::Success,
                        );
                    }
                    Err(e) => {
                        return window
                            .set_status(format!("Failed to delete: {}", e), StatusSeverity::Error);
                    }
                }
            }
            Task::none()
        }
        Message::ConfirmLoadProfile => {
            let name = match &window.editor_state.session.pending_confirm {
                ConfirmAction::LoadProfile { name } => name.clone(),
                _ => {
                    window.editor_state.session.pending_confirm = ConfirmAction::None;
                    return Task::none();
                }
            };
            window.editor_state.session.pending_confirm = ConfirmAction::None;
            window.editor_state.push_undo();

            let (profile_name, was_truncated) = match window
                .editor_state
                .ui
                .profiles
                .iter()
                .find(|p| p.name == name)
            {
                Some(profile) => {
                    let profile_name = profile.name.clone();
                    let (was_truncated, _) = apply_peq_to_editor(window, profile.data.clone());
                    window.editor_state.ui.selected_profile_name = Some(name.clone());
                    window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;
                    window.editor_state.session.new_profile_name = profile_name.clone();
                    window.editor_state.ui.profile_search.clear();
                    window.editor_state.session.is_autoeq_active = false;
                    (profile_name, was_truncated)
                }
                None => return Task::none(),
            };

            if was_truncated {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Warn,
                    Source::UI,
                    format!(
                        "Profile {} truncated to {} bands",
                        profile_name,
                        window.num_bands()
                    ),
                ));
                window.set_status(
                    format!(
                        "Loaded profile: {} (truncated to {})",
                        profile_name,
                        window.num_bands()
                    ),
                    StatusSeverity::Warning,
                )
            } else {
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    format!("Loaded profile: {}", profile_name),
                ));
                window.set_status(
                    format!("Loaded profile: {}", profile_name),
                    StatusSeverity::Info,
                )
            }
        }
        Message::ImportFromFilePressed => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Frost-Tune Profile", &["json", "txt"])
                    .pick_file()
                    .await
            },
            |handle| Message::FileImported(handle.map(|h| h.path().to_path_buf())),
        ),
        Message::FileImported(path_opt) => {
            if let Some(path) = path_opt {
                match crate::storage::import_profile(&path) {
                    Ok(profile) => {
                        window.editor_state.session.pending_confirm = ConfirmAction::ImportAutoEQ {
                            data: profile.data,
                            default_name: profile.name,
                        };
                        Task::none()
                    }
                    Err(e) => {
                        window.set_status(format!("Import failed: {}", e), StatusSeverity::Error)
                    }
                }
            } else {
                Task::none()
            }
        }
        Message::ExportToFilePressed => {
            let peq = PEQData {
                filters: window.editor_state.data.filters.clone(),
                global_gain: window.editor_state.data.global_gain,
            };
            let name = if window.editor_state.session.new_profile_name.is_empty() {
                "profile".to_string()
            } else {
                window.editor_state.session.new_profile_name.clone()
            };

            Task::perform(
                async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Frost-Tune Profile", &["json", "txt"])
                        .set_file_name(format!("{}.txt", name))
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
                    Err(e) => {
                        window.set_status(format!("Export failed: {}", e), StatusSeverity::Error)
                    }
                }
            } else {
                Task::none()
            }
        }
        Message::ProfileSearchInput(query) => {
            window.editor_state.ui.profile_search = query;
            Task::none()
        }
        Message::ToolsTabSelected(tab) => {
            window.editor_state.ui.active_tools_tab = tab;
            Task::none()
        }
        _ => Task::none(),
    }
}
