// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::PEQData;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::error::AppError;
use crate::ui::components::editor::ConfirmAction;
use crate::ui::messages::*;
use crate::ui::state::MainWindow;
use iced::Task;

fn reload_profiles_task() -> Task<Message> {
    Task::perform(
        async move { crate::storage::load_all_profiles().await },
        |res| Message::Profiles(ProfilesMessage::ProfilesLoaded(res)),
    )
}

pub(crate) fn apply_peq_to_editor(window: &mut MainWindow, mut peq: PEQData) -> (bool, usize) {
    if let Some(active) = window.active_device() {
        peq.clamp_to_capabilities(&active.capabilities());
    } else {
        peq.clamp_to_capabilities(&crate::core::device::capabilities::DESKTOP_DAC_CAPS);
    }

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
    window.editor.data.filters = filters
        .into_iter()
        .enumerate()
        .map(|(i, mut f)| {
            f.index = i as u8;
            f.enabled = true;
            f.clamp(freq_range, gain_range, q_range);
            f
        })
        .collect();

    while window.editor.data.filters.len() < num_bands {
        window
            .editor
            .data
            .filters
            .push(crate::core::Filter::enabled(
                window.editor.data.filters.len() as u8,
                false,
            ));
    }

    window.editor.data.global_gain = peq.global_gain;
    window.editor.session.is_autoeq_active = true;

    (was_truncated, enabled_count)
}

fn check_overwrite_and_save(
    window: &mut MainWindow,
    name: String,
    data: PEQData,
    context: crate::ui::messages::SaveContext,
) -> Task<Message> {
    let name_exists = window.editor.ui.profiles.iter().any(|p| p.name == name);

    if name_exists {
        window.editor.session.pending_confirm = ConfirmAction::OverwriteProfile { name, data };
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
                .await
                .map_err(|e| AppError::new(crate::error::ErrorKind::StorageError, e.to_string()))
        },
        move |result| {
            Message::Profiles(ProfilesMessage::ProfileSaved {
                name: name.clone(),
                data: data.clone(),
                result,
                context: context.clone(),
            })
        },
    )
}

pub fn handle_profiles(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::Profiles(ProfilesMessage::ReloadProfilesPressed) => reload_profiles_task(),
        Message::Profiles(ProfilesMessage::OpenProfilesDirPressed) => {
            if let Err(e) = crate::storage::open_profiles_dir() {
                window.set_status(
                    format!("Failed to open folder: {}", e),
                    StatusSeverity::Error,
                )
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::ProfilesLoaded(result)) => {
            window.editor.ui.profiles_dir_mtime = crate::storage::get_profiles_dir_mtime();
            match result {
                Ok((profiles, errors)) => {
                    let prev_count = window.editor.ui.profiles.len();
                    window.editor.ui.profiles = profiles;

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
                                window.editor.ui.profiles.len(),
                                errors.len()
                            ),
                            StatusSeverity::Warning,
                        )
                    } else if window.editor.ui.profiles.len() != prev_count {
                        window.set_status(
                            format!(
                                "Profiles updated ({} total)",
                                window.editor.ui.profiles.len()
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
        Message::Profiles(ProfilesMessage::ProfileSelected(name)) => {
            if window.editor.session.is_dirty {
                window.editor.session.pending_confirm =
                    crate::ui::components::editor::ConfirmAction::LoadProfile { name };
                return Task::none();
            }

            window.editor.push_undo();
            let (profile_name, was_truncated) =
                match window.editor.ui.profiles.iter().find(|p| p.name == name) {
                    Some(profile) => {
                        let profile_name = profile.name.clone();
                        let (was_truncated, _) = apply_peq_to_editor(window, profile.data.clone());
                        window.editor.ui.selected_profile_name = Some(name);
                        window.editor.ui.eq_source =
                            crate::ui::components::editor::EqSource::Profile;
                        window.editor.session.new_profile_name = profile_name.clone();
                        window.editor.ui.profile_search.clear();
                        window.editor.session.is_autoeq_active = false;
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
                window.set_status_silent(
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
                window.set_status_silent(
                    format!("Loaded profile: {}", profile_name),
                    StatusSeverity::Info,
                )
            }
        }
        Message::Profiles(ProfilesMessage::ProfileNameInput(name)) => {
            window.editor.session.new_profile_name = name;
            Task::none()
        }
        Message::AutoEq(AutoEqMessage::ImportNameInput(name)) => {
            window.editor.session.import_name_input = name;
            Task::none()
        }
        Message::AutoEq(AutoEqMessage::ImportProfileSelected(name)) => {
            window.editor.session.import_name_input = name;
            Task::none()
        }
        Message::AutoEq(AutoEqMessage::ImportTemporaryToggled(val)) => {
            window.editor.session.import_temporary = val;
            Task::none()
        }
        Message::AutoEq(AutoEqMessage::ImportDirectlyToEditor) => {
            if let ConfirmAction::ImportAutoEQ { data, .. } =
                window.editor.session.pending_confirm.clone()
            {
                window.editor.push_undo();
                let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                window.editor.session.import_name_input = String::new();
                window.editor.session.pending_confirm = ConfirmAction::None;
                window.editor.session.is_dirty = true;
                window.editor.ui.selected_profile_name = None;
                window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Imported;

                let mut tasks = vec![];

                if was_truncated {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Warn,
                        Source::UI,
                        format!("Import truncated to {} bands", window.num_bands()),
                    ));
                    tasks.push(window.set_status_silent(
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
                    tasks.push(window.set_status_silent(
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
        Message::AutoEq(AutoEqMessage::ImportOverwriteActive) => {
            if let ConfirmAction::ImportAutoEQ { data, .. } =
                window.editor.session.pending_confirm.clone()
            {
                if let Some(name) = window.editor.ui.selected_profile_name.clone() {
                    do_save_profile(window, name, data, SaveContext::ImportOverwrite)
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::SaveProfilePressed) => {
            let name = window.editor.session.new_profile_name.trim().to_string();
            if name.is_empty() {
                return window.set_status("Invalid profile name", StatusSeverity::Warning);
            }
            let data = PEQData {
                filters: window.editor.data.filters.clone(),
                global_gain: window.editor.data.global_gain,
            };
            check_overwrite_and_save(window, name, data, SaveContext::Standard)
        }
        Message::AutoEq(AutoEqMessage::ConfirmImportWithName) => {
            if let ConfirmAction::ImportAutoEQ { data, default_name } =
                window.editor.session.pending_confirm.clone()
            {
                let typed = window.editor.session.import_name_input.trim();
                let name = if typed.is_empty() {
                    default_name.trim().to_string()
                } else {
                    typed.to_string()
                };

                if name.is_empty() {
                    return window
                        .set_status("Profile name cannot be empty", StatusSeverity::Warning);
                }

                let name_exists = window.editor.ui.profiles.iter().any(|p| p.name == name);

                if name_exists {
                    window.editor.session.pending_confirm =
                        ConfirmAction::OverwriteProfile { name, data };
                    return Task::none();
                }

                do_save_profile(window, name, data, SaveContext::ImportWithName)
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::ConfirmOverwriteProfile) => {
            if let ConfirmAction::OverwriteProfile { name, data } =
                window.editor.session.pending_confirm.clone()
            {
                do_save_profile(window, name, data, SaveContext::Overwrite)
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::DeleteProfilePressed) => {
            window.editor.session.pending_confirm = ConfirmAction::DeleteProfile;
            Task::none()
        }
        Message::Profiles(ProfilesMessage::ConfirmDeleteProfile) => {
            if matches!(
                window.editor.session.pending_confirm,
                ConfirmAction::DeleteProfile
            ) {
                window.editor.session.pending_confirm = ConfirmAction::None;
            } else {
                return Task::none();
            }

            if let Some(name) = window.editor.ui.selected_profile_name.clone() {
                let name_clone = name.clone();
                Task::perform(
                    async move {
                        crate::storage::delete_profile(&name_clone)
                            .await
                            .map_err(|e| {
                                AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                            })
                    },
                    move |result| {
                        Message::Profiles(ProfilesMessage::ProfileDeleted {
                            name: name.clone(),
                            result,
                        })
                    },
                )
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::ConfirmLoadProfile) => {
            let name = match &window.editor.session.pending_confirm {
                ConfirmAction::LoadProfile { name } => name.clone(),
                _ => {
                    window.editor.session.pending_confirm = ConfirmAction::None;
                    return Task::none();
                }
            };
            window.editor.session.pending_confirm = ConfirmAction::None;
            window.editor.push_undo();

            let (profile_name, was_truncated) =
                match window.editor.ui.profiles.iter().find(|p| p.name == name) {
                    Some(profile) => {
                        let profile_name = profile.name.clone();
                        let (was_truncated, _) = apply_peq_to_editor(window, profile.data.clone());
                        window.editor.ui.selected_profile_name = Some(name.clone());
                        window.editor.ui.eq_source =
                            crate::ui::components::editor::EqSource::Profile;
                        window.editor.session.new_profile_name = profile_name.clone();
                        window.editor.ui.profile_search.clear();
                        window.editor.session.is_autoeq_active = false;
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
                window.set_status_silent(
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
                window.set_status_silent(
                    format!("Loaded profile: {}", profile_name),
                    StatusSeverity::Info,
                )
            }
        }
        Message::Profiles(ProfilesMessage::ImportFromFilePressed) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("AutoEQ Profile", &["txt"])
                    .pick_file()
                    .await
            },
            |handle| {
                Message::Profiles(ProfilesMessage::FileImported(
                    handle.map(|h| h.path().to_path_buf()),
                ))
            },
        ),
        Message::Profiles(ProfilesMessage::FileImported(path_opt)) => {
            if let Some(path) = path_opt {
                let ext = path.extension().and_then(|e| e.to_str());
                if ext != Some("txt") {
                    let _ = window.set_status(
                        "Unsupported file type. Only .txt AutoEQ files are supported.",
                        StatusSeverity::Error,
                    );
                    return Task::none();
                }
                if window.editor.session.pending_confirm != ConfirmAction::None {
                    return Task::none();
                }
                Task::perform(
                    async move {
                        crate::storage::import_profile(&path).await.map_err(|e| {
                            AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                        })
                    },
                    |result| Message::Profiles(ProfilesMessage::ProfileImported { result }),
                )
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::ExportToFilePressed) => {
            let peq = PEQData {
                filters: window.editor.data.filters.clone(),
                global_gain: window.editor.data.global_gain,
            };
            let name = if window.editor.session.new_profile_name.is_empty() {
                "profile".to_string()
            } else {
                window.editor.session.new_profile_name.clone()
            };

            Task::perform(
                async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("AutoEQ Profile", &["txt"])
                        .set_file_name(format!("{}.txt", name))
                        .save_file()
                        .await
                },
                move |handle| {
                    Message::Profiles(ProfilesMessage::FileExported(
                        handle.map(|h| h.path().to_path_buf()),
                        peq.clone(),
                    ))
                },
            )
        }
        Message::Profiles(ProfilesMessage::FileExported(path_opt, peq)) => {
            if let Some(path) = path_opt {
                Task::perform(
                    async move {
                        crate::storage::export_profile(&path, &peq)
                            .await
                            .map_err(|e| {
                                AppError::new(crate::error::ErrorKind::StorageError, e.to_string())
                            })
                    },
                    |result| Message::Profiles(ProfilesMessage::ProfileExported { result }),
                )
            } else {
                Task::none()
            }
        }
        Message::Profiles(ProfilesMessage::ProfileSearchInput(query)) => {
            window.editor.ui.profile_search = query;
            Task::none()
        }
        Message::ToolsTabSelected(tab) => {
            window.editor.ui.active_tools_tab = tab;
            Task::none()
        }
        Message::Profiles(ProfilesMessage::ProfileSaved {
            name,
            data,
            result,
            context,
        }) => match result {
            Ok(_) => match context {
                SaveContext::Standard => {
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Saved profile: {}", name),
                    ));
                    window.editor.session.new_profile_name = name.clone();
                    window.editor.ui.selected_profile_name = Some(name.clone());
                    window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Profile;
                    window.editor.session.is_dirty = false;
                    Task::batch(vec![
                        reload_profiles_task(),
                        window.set_status_silent(
                            format!("Saved profile: {}", name),
                            StatusSeverity::Success,
                        ),
                    ])
                }
                SaveContext::ImportOverwrite => {
                    window.editor.push_undo();
                    let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                    window.editor.session.import_name_input = String::new();
                    window.editor.session.pending_confirm = ConfirmAction::None;
                    window.editor.session.is_dirty = false;
                    window.editor.session.new_profile_name = name.clone();
                    window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Profile;

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
                        tasks.push(window.set_status_silent(
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
                        tasks.push(window.set_status_silent(
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
                    window.editor.push_undo();
                    let (was_truncated, enabled_count) = apply_peq_to_editor(window, data);
                    window.editor.session.import_name_input = String::new();
                    window.editor.session.pending_confirm = ConfirmAction::None;
                    window.editor.ui.selected_profile_name = Some(name.clone());
                    window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Profile;

                    let mut tasks = vec![reload_profiles_task()];

                    if was_truncated {
                        window.diagnostics.push(DiagnosticEvent::new(
                            LogLevel::Warn,
                            Source::UI,
                            format!("Import truncated to {} bands", window.num_bands()),
                        ));
                        tasks.push(window.set_status_silent(
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
                        tasks.push(window.set_status_silent(
                            format!("Imported {} filters", enabled_count),
                            StatusSeverity::Success,
                        ));
                    }
                    Task::batch(tasks)
                }
                SaveContext::Overwrite => {
                    window.editor.push_undo();
                    apply_peq_to_editor(window, data);
                    window.editor.session.pending_confirm = ConfirmAction::None;
                    window.editor.session.new_profile_name = name.clone();
                    window.editor.ui.selected_profile_name = Some(name.clone());
                    window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Profile;
                    window.diagnostics.push(DiagnosticEvent::new(
                        LogLevel::Info,
                        Source::UI,
                        format!("Overwritten profile: {}", name),
                    ));
                    let reload_task = reload_profiles_task();
                    let status_task = window.set_status_silent(
                        format!("Overwritten profile: {}", name),
                        StatusSeverity::Success,
                    );
                    Task::batch(vec![reload_task, status_task])
                }
                SaveContext::Exit(id) => {
                    window.editor.session.is_dirty = false;
                    window.editor.session.pending_confirm = ConfirmAction::None;
                    iced::window::close(id)
                }
                SaveContext::LoadProfile(profile_to_load) => {
                    window.editor.session.is_dirty = false;
                    let load_task = Task::perform(async move { profile_to_load }, |name| {
                        Message::Profiles(ProfilesMessage::ProfileSelected(name))
                    });
                    Task::batch(vec![reload_profiles_task(), load_task])
                }
            },
            Err(e) => {
                let action_str = match context {
                    SaveContext::Standard => "save",
                    SaveContext::ImportOverwrite => "overwrite",
                    SaveContext::ImportWithName => "import",
                    SaveContext::Overwrite => "save",
                    SaveContext::Exit(_) => "save on exit",
                    SaveContext::LoadProfile(_) => "save before loading",
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
        Message::Profiles(ProfilesMessage::ProfileDeleted { name, result }) => match result {
            Ok(_) => {
                window.editor.session.is_dirty = false;
                window.editor.ui.selected_profile_name = None;
                window.editor.ui.eq_source = crate::ui::components::editor::EqSource::Default;
                window.editor.session.new_profile_name.clear();
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    format!("Deleted profile: {}", name),
                ));
                Task::batch(vec![
                    reload_profiles_task(),
                    window.set_status_silent(
                        format!("Deleted profile: {}", name),
                        StatusSeverity::Success,
                    ),
                ])
            }
            Err(e) => window.set_status(format!("Failed to delete: {}", e), StatusSeverity::Error),
        },
        Message::Profiles(ProfilesMessage::ProfileImported { result }) => match result {
            Ok(mut profile) => {
                if let Some(active) = window.active_device() {
                    profile.data.clamp_to_capabilities(&active.capabilities());
                } else {
                    profile.data.clamp_to_capabilities(
                        &crate::core::device::capabilities::DESKTOP_DAC_CAPS,
                    );
                }

                window.editor.session.import_temporary = false;
                window.editor.session.import_name_input.clear();
                window.editor.session.pending_confirm = ConfirmAction::ImportAutoEQ {
                    data: profile.data,
                    default_name: profile.name,
                };
                Task::none()
            }
            Err(e) => window.set_status(format!("Import failed: {}", e), StatusSeverity::Error),
        },
        Message::Profiles(ProfilesMessage::ProfileExported { result }) => match result {
            Ok(_) => window.set_status("Profile exported", StatusSeverity::Success),
            Err(e) => window.set_status(format!("Export failed: {}", e), StatusSeverity::Error),
        },
        _ => Task::none(),
    }
}
