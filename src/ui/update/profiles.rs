// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::error::AppError;
use crate::models::PEQData;
use crate::ui::messages::{Message, SaveContext, StatusSeverity};
use crate::ui::state::{ConfirmAction, MainWindow};
use iced::Task;

fn reload_profiles_task() -> Task<Message> {
    Task::perform(
        async move { crate::storage::load_all_profiles() },
        Message::ProfilesLoaded,
    )
}

pub(crate) fn apply_peq_to_editor(window: &mut MainWindow, peq: PEQData) -> (bool, usize) {
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
    context: crate::ui::messages::SaveContext,
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

    do_save_profile(window, name, data, context)
}

fn do_save_profile(
    _window: &mut MainWindow,
    name: String,
    data: PEQData,
    context: crate::ui::messages::SaveContext,
) -> Task<Message> {
    let name_clone = name.clone();
    let data_clone = data.clone();
    Task::perform(
        async move {
            crate::storage::save_profile(&name_clone, &data_clone)
                .map_err(|e| AppError::new(crate::error::ErrorKind::StorageError, e.to_string()))
        },
        move |result| Message::ProfileSaved {
            name: name.clone(),
            data: data.clone(),
            result,
            context: context.clone(),
        },
    )
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
        Message::ImportProfileSelected(name) => {
            window.editor_state.session.import_name_input = name;
            Task::none()
        }
        Message::ImportTemporaryToggled(val) => {
            window.editor_state.session.import_temporary = val;
            Task::none()
        }
        Message::ImportDirectlyToEditor => {
            if let ConfirmAction::ImportAutoEQ { data, .. } =
                window.editor_state.session.pending_confirm.clone()
            {
                window.editor_state.push_undo();
                let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                window.editor_state.session.import_name_input = String::new();
                window.editor_state.session.pending_confirm = ConfirmAction::None;
                window.editor_state.session.is_dirty = true;
                window.editor_state.ui.selected_profile_name = None;
                window.editor_state.ui.eq_source = crate::ui::state::EqSource::Imported;

                let mut tasks = vec![];

                if was_truncated {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::UI,
                        format!("Import truncated to {} bands", window.num_bands()),
                    ));
                    tasks.push(window.set_status(
                        format!(
                            "Imported {} filters to current EQ (truncated to {})",
                            enabled_count,
                            window.num_bands()
                        ),
                        StatusSeverity::Warning,
                    ));
                } else {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Applied {} filters directly to editor", enabled_count),
                    ));
                    tasks.push(window.set_status(
                        format!(
                            "Applied {} filters directly to current EQ (unsaved)",
                            enabled_count
                        ),
                        StatusSeverity::Success,
                    ));
                }
                Task::batch(tasks)
            } else {
                Task::none()
            }
        }
        Message::ImportOverwriteActive => {
            if let ConfirmAction::ImportAutoEQ { data, .. } =
                window.editor_state.session.pending_confirm.clone()
            {
                if let Some(name) = window.editor_state.ui.selected_profile_name.clone() {
                    do_save_profile(window, name, data, SaveContext::ImportOverwrite)
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
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
            check_overwrite_and_save(window, name, data, SaveContext::Standard)
        }
        Message::ConfirmImportWithName => {
            if let ConfirmAction::ImportAutoEQ { data, default_name } =
                window.editor_state.session.pending_confirm.clone()
            {
                let typed = window.editor_state.session.import_name_input.trim();
                let name = if typed.is_empty() {
                    default_name.trim().to_string()
                } else {
                    typed.to_string()
                };

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

                do_save_profile(window, name, data, SaveContext::ImportWithName)
            } else {
                Task::none()
            }
        }
        Message::ConfirmOverwriteProfile => {
            if let ConfirmAction::OverwriteProfile { name, data } =
                window.editor_state.session.pending_confirm.clone()
            {
                do_save_profile(window, name, data, SaveContext::Overwrite)
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
                let name_clone = name.clone();
                Task::perform(
                    async move {
                        crate::storage::delete_profile(&name_clone).map_err(|e| {
                            AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                        })
                    },
                    move |result| Message::ProfileDeleted {
                        name: name.clone(),
                        result,
                    },
                )
            } else {
                Task::none()
            }
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
                    .add_filter("AutoEQ Profile", &["txt"])
                    .pick_file()
                    .await
            },
            |handle| Message::FileImported(handle.map(|h| h.path().to_path_buf())),
        ),
        Message::FileImported(path_opt) => {
            if let Some(path) = path_opt {
                let ext = path.extension().and_then(|e| e.to_str());
                if ext != Some("txt") {
                    let _ = window.set_status(
                        "Unsupported file type. Only .txt AutoEQ files are supported.",
                        StatusSeverity::Error,
                    );
                    return Task::none();
                }
                if window.editor_state.session.pending_confirm != ConfirmAction::None {
                    return Task::none();
                }
                Task::perform(
                    async move {
                        crate::storage::import_profile(&path).map_err(|e| {
                            AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                        })
                    },
                    |result| Message::ProfileImported { result },
                )
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
                        .add_filter("AutoEQ Profile", &["txt"])
                        .set_file_name(format!("{}.txt", name))
                        .save_file()
                        .await
                },
                move |handle| Message::FileExported(handle.map(|h| h.path().to_path_buf()), peq),
            )
        }
        Message::FileExported(path_opt, peq) => {
            if let Some(path) = path_opt {
                Task::perform(
                    async move {
                        crate::storage::export_profile(&path, &peq).map_err(|e| {
                            AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                        })
                    },
                    |result| Message::ProfileExported { result },
                )
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
        Message::ProfileSaved {
            name,
            data,
            result,
            context,
        } => match result {
            Ok(_) => match context {
                SaveContext::Standard => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Saved profile: {}", name),
                    ));
                    window.editor_state.session.new_profile_name = name.clone();
                    window.editor_state.ui.selected_profile_name = Some(name.clone());
                    window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;
                    window.editor_state.session.is_dirty = false;
                    Task::batch(vec![
                        reload_profiles_task(),
                        window.set_status(
                            format!("Saved profile: {}", name),
                            StatusSeverity::Success,
                        ),
                    ])
                }
                SaveContext::ImportOverwrite => {
                    window.editor_state.push_undo();
                    let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                    window.editor_state.session.import_name_input = String::new();
                    window.editor_state.session.pending_confirm = ConfirmAction::None;
                    window.editor_state.session.is_dirty = false;
                    window.editor_state.session.new_profile_name = name.clone();
                    window.editor_state.ui.eq_source = crate::ui::state::EqSource::Profile;

                    let mut tasks = vec![reload_profiles_task()];

                    if was_truncated {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Warn,
                            Source::UI,
                            format!(
                                "Profile '{}' updated but truncated to {} bands",
                                name,
                                window.num_bands()
                            ),
                        ));
                        tasks.push(window.set_status(
                            format!(
                                "Updated '{}' with {} filters (truncated to {})",
                                name,
                                enabled_count,
                                window.num_bands()
                            ),
                            StatusSeverity::Warning,
                        ));
                    } else {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Info,
                            Source::UI,
                            format!("Profile '{}' updated with {} filters", name, enabled_count),
                        ));
                        tasks.push(window.set_status(
                            format!(
                                "Overwrote profile '{}' with {} filters",
                                name, enabled_count
                            ),
                            StatusSeverity::Success,
                        ));
                    }
                    Task::batch(tasks)
                }
                SaveContext::ImportWithName => {
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
                SaveContext::Overwrite => {
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
                SaveContext::Exit(id) => {
                    window.editor_state.session.is_dirty = false;
                    window.editor_state.session.pending_confirm = ConfirmAction::None;
                    iced::window::close(id)
                }
            },
            Err(e) => {
                let action_str = match context {
                    SaveContext::Standard => "save",
                    SaveContext::ImportOverwrite => "overwrite",
                    SaveContext::ImportWithName => "import",
                    SaveContext::Overwrite => "save",
                    SaveContext::Exit(_) => "save on exit",
                };
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Error,
                    Source::UI,
                    format!("Save failed: {}", e),
                ));
                window.set_status(
                    format!("Failed to {}: {}", action_str, e),
                    StatusSeverity::Error,
                )
            }
        },
        Message::ProfileDeleted { name, result } => match result {
            Ok(_) => {
                window.editor_state.session.is_dirty = false;
                window.editor_state.ui.selected_profile_name = None;
                window.editor_state.ui.eq_source = crate::ui::state::EqSource::Default;
                window.editor_state.session.new_profile_name.clear();
                Task::batch(vec![
                    reload_profiles_task(),
                    window.set_status(
                        format!("Deleted profile: {}", name),
                        StatusSeverity::Success,
                    ),
                ])
            }
            Err(e) => window.set_status(format!("Failed to delete: {}", e), StatusSeverity::Error),
        },
        Message::ProfileImported { result } => match result {
            Ok(profile) => {
                window.editor_state.session.import_temporary = false;
                window.editor_state.session.import_name_input.clear();
                window.editor_state.session.pending_confirm = ConfirmAction::ImportAutoEQ {
                    data: profile.data,
                    default_name: profile.name,
                };
                Task::none()
            }
            Err(e) => window.set_status(format!("Import failed: {}", e), StatusSeverity::Error),
        },
        Message::ProfileExported { result } => match result {
            Ok(_) => window.set_status("Profile exported", StatusSeverity::Success),
            Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
        },
        _ => Task::none(),
    }
}
