// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::*;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::ui::components::editor::{ConfirmAction, DraftFilter};
use crate::ui::main_window::parse_freq_string;
use crate::ui::messages::*;
use crate::ui::state::AppState;
use iced::Task;

fn handle_band_text_input(
    window: &mut AppState,
    index: usize,
    s: String,
    setter: impl FnOnce(&mut DraftFilter, String),
) {
    let filter = match window.editor.data.filters.get(index) {
        Some(f) => f,
        None => {
            log::error!("Band input: index {} out of bounds", index);
            return;
        }
    };
    let draft = window
        .editor
        .session
        .input_buffer
        .active_draft
        .get_or_insert_with(|| DraftFilter::from_filter(filter));
    if draft.index != index {
        *draft = DraftFilter::from_filter(filter);
    }
    setter(draft, s);
}

fn cancel_band_draft_input(window: &mut AppState, index: usize) {
    if let Some(draft) = window.editor.session.input_buffer.active_draft.take() {
        if draft.index != index {
            window.editor.session.input_buffer.active_draft = Some(draft);
        }
    }
}

fn commit_band_field(
    window: &mut AppState,
    index: usize,
    parse_and_apply: impl FnOnce(&mut Filter, &mut DraftFilter) -> bool,
) -> Task<Message> {
    if let Some(mut draft) = window.editor.session.input_buffer.active_draft.take() {
        if draft.index == index {
            window.editor.push_undo();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                if parse_and_apply(band, &mut draft) {
                    window.editor.session.is_dirty = true;
                } else {
                    window.editor.session.input_buffer.active_draft = Some(draft);
                }
            }
        } else {
            window.editor.session.input_buffer.active_draft = Some(draft);
        }
    }
    Task::none()
}

pub fn handle_editor(window: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Editor(EditorMessage::BandFreqChanged(index, freq)) => {
            window.editor.push_undo();
            let freq_range = window.freq_range();
            let gain_range = window.gain_range();
            let q_range = window.q_range();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                band.freq = freq;
                band.enabled = true;
                band.clamp(freq_range, gain_range, q_range);
                window.editor.session.is_dirty = true;
            }
            Task::none()
        }
        Message::Editor(EditorMessage::BandTypeChanged(index, t)) => {
            window.editor.push_undo();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                band.filter_type = t;
                window.editor.session.is_dirty = true;
            }
            Task::none()
        }
        Message::Editor(EditorMessage::BandEnabledToggled(index, en)) => {
            if !window.supports_per_band_enable() {
                return Task::none();
            }
            window.editor.push_undo();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                band.enabled = en;
                window.editor.session.is_dirty = true;
            }
            Task::none()
        }
        Message::Editor(EditorMessage::BandFreqInput(index, s)) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.freq_input = val;
                draft.freq_error = None;
            });
            Task::none()
        }
        Message::Editor(EditorMessage::BandGainInput(index, s)) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.gain_input = val;
                draft.gain_error = None;
            });
            Task::none()
        }
        Message::Editor(EditorMessage::BandQInput(index, s)) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.q_input = val;
                draft.q_error = None;
            });
            Task::none()
        }
        Message::Editor(EditorMessage::BandFreqInputCommit(index)) => {
            let (min, max) = window.freq_range();
            commit_band_field(window, index, move |band, draft| {
                if let Some(v) = parse_freq_string(&draft.freq_input) {
                    band.freq = v.clamp(min, max);
                    band.enabled = true;
                    true
                } else {
                    draft.freq_error = Some(format!("Freq: {}-{} Hz", min, max));
                    false
                }
            })
        }
        Message::Editor(EditorMessage::BandGainInputCommit(index)) => {
            let (min, max) = window.gain_range();
            commit_band_field(window, index, move |band, draft| {
                match draft.gain_input.trim().parse::<f64>() {
                    Ok(v) if v >= min && v <= max => {
                        band.gain = v;
                        band.enabled = true;
                        true
                    }
                    Ok(_) => {
                        draft.gain_error = Some(format!("Gain: {:.0} to {:.0}", min, max));
                        false
                    }
                    Err(_) => {
                        draft.gain_error = Some("Gain: enter number".to_string());
                        false
                    }
                }
            })
        }
        Message::Editor(EditorMessage::BandQInputCommit(index)) => {
            let (min, max) = window.q_range();
            commit_band_field(window, index, move |band, draft| {
                match draft.q_input.trim().parse::<f64>() {
                    Ok(v) if v >= min && v <= max => {
                        band.q = v;
                        band.enabled = true;
                        true
                    }
                    Ok(_) => {
                        draft.q_error = Some(format!("Q: {:.1} to {:.1}", min, max));
                        false
                    }
                    Err(_) => {
                        draft.q_error = Some("Q: enter number".to_string());
                        false
                    }
                }
            })
        }
        Message::Editor(EditorMessage::BandFreqInputCancel(index)) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::Editor(EditorMessage::BandGainInputCancel(index)) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::Editor(EditorMessage::BandQInputCancel(index)) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::Editor(EditorMessage::BandFreqSliderChanged(index, v)) => {
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                let hz = 10f64.powf(v).round() as u16;
                band.freq = if window.editor.ui.snap_to_iso_enabled {
                    snap_freq_to_iso(hz)
                } else {
                    hz
                };
                window.editor.session.is_dirty = true;
                if let Some(draft) = window.editor.session.input_buffer.active_draft.as_mut() {
                    if draft.index == index {
                        draft.freq_input = band.freq.to_string();
                    }
                }
            }
            Task::none()
        }
        Message::Editor(EditorMessage::BandFreqSliderReleased(_index)) => {
            window.editor.push_undo();
            Task::none()
        }
        Message::Editor(EditorMessage::BandGainChanged(index, v)) => {
            let (min_gain, max_gain) = window.gain_range();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                band.gain = v.clamp(min_gain, max_gain);
                band.enabled = true;
                window.editor.session.is_dirty = true;
                if let Some(draft) = window.editor.session.input_buffer.active_draft.as_mut() {
                    if draft.index == index {
                        draft.gain_input = format!("{:.2}", band.gain);
                    }
                }
            }
            Task::none()
        }
        Message::Editor(EditorMessage::BandGainReleased(_index)) => {
            window.editor.push_undo();
            Task::none()
        }
        Message::Editor(EditorMessage::BandQChanged(index, v)) => {
            window.editor.push_undo();
            if let Some(band) = window.editor.data.filters.get_mut(index) {
                let q_val = 10f64.powf(v);
                band.q = snap_q_to_iso(q_val);
                window.editor.session.is_dirty = true;
                if let Some(draft) = window.editor.session.input_buffer.active_draft.as_mut() {
                    if draft.index == index {
                        draft.q_input = format!("{:.2}", band.q);
                    }
                }
            }
            Task::none()
        }
        Message::Editor(EditorMessage::GlobalGainChanged(gain)) => {
            window.editor.push_undo();
            let gain_range = window.global_gain_range();
            window.editor.data.global_gain = gain.clamp(*gain_range.start(), *gain_range.end());
            window.editor.session.is_dirty = true;
            Task::none()
        }
        Message::Editor(EditorMessage::ResetFiltersPressed) => {
            window.editor.session.pending_confirm = ConfirmAction::ResetFilters;
            Task::none()
        }
        Message::Editor(EditorMessage::ConfirmResetFilters) => {
            let num_bands = window.num_bands();
            if matches!(
                window.editor.session.pending_confirm,
                ConfirmAction::ResetFilters
            ) {
                window.editor.push_undo();
                window.editor.data.filters.clear();
                for i in 0..num_bands {
                    window
                        .editor
                        .data
                        .filters
                        .push(Filter::enabled(i as u8, false));
                }
                window.editor.data.global_gain = 0;
                window.editor.session.is_dirty = true;
                window.editor.session.is_autoeq_active = false;
                window.editor.session.input_buffer.active_draft = None;
                window.editor.ui.selected_profile_name = None;
                window.editor.ui.eq_source = crate::ui::messages::EqSource::Default;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Reset filters to default",
                ));
                window.editor.session.pending_confirm = ConfirmAction::None;
                window.set_status("Filters reset to default", StatusSeverity::Info)
            } else {
                Task::none()
            }
        }
        Message::Diagnostics(DiagnosticsMessage::ToggleDiagnostics) => {
            window.editor.ui.show_diagnostics = !window.editor.ui.show_diagnostics;
            Task::none()
        }

        Message::Editor(EditorMessage::Undo) => {
            if let Some(prev) = window.editor.session.undo_stack.pop() {
                let current = std::sync::Arc::new(crate::core::PEQData {
                    filters: window.editor.data.filters.clone(),
                    global_gain: window.editor.data.global_gain,
                });
                window.editor.session.redo_stack.push(current);
                window.editor.data.filters = prev.filters.clone();
                window.editor.data.global_gain = prev.global_gain;
                window.editor.data.generation += 1;
                window.editor.session.is_dirty = true;
                window.editor.session.input_buffer.active_draft = None;
            }
            Task::none()
        }
        Message::Editor(EditorMessage::Redo) => {
            if let Some(next) = window.editor.session.redo_stack.pop() {
                let current = std::sync::Arc::new(crate::core::PEQData {
                    filters: window.editor.data.filters.clone(),
                    global_gain: window.editor.data.global_gain,
                });
                window.editor.session.undo_stack.push(current);
                window.editor.data.filters = next.filters.clone();
                window.editor.data.global_gain = next.global_gain;
                window.editor.data.generation += 1;
                window.editor.session.is_dirty = true;
                window.editor.session.input_buffer.active_draft = None;
            }
            Task::none()
        }

        Message::Editor(EditorMessage::ToggleSnapToIso(enabled)) => {
            window.editor.ui.snap_to_iso_enabled = enabled;
            Task::none()
        }
        Message::Editor(EditorMessage::ToggleAutoPullOnConnect(enabled)) => {
            window.editor.ui.auto_pull_on_connect = enabled;
            // Persist immediately; ignore I/O errors so the toggle still flips in-memory.
            Task::perform(
                async move {
                    let settings = crate::storage::load_settings();
                    crate::storage::save_settings(crate::storage::Settings {
                        auto_pull_on_connect: enabled,
                        skip_push_verification: settings.skip_push_verification,
                    })
                    .map_err(|e| {
                        crate::error::AppError::new(
                            crate::error::ErrorKind::StorageError,
                            e.to_string(),
                        )
                    })
                },
                |result| Message::Editor(EditorMessage::SettingsSaved { result }),
            )
        }
        Message::Editor(EditorMessage::ToggleSkipPushVerification(enabled)) => {
            window.editor.ui.skip_push_verification = enabled;
            Task::perform(
                async move {
                    let settings = crate::storage::load_settings();
                    crate::storage::save_settings(crate::storage::Settings {
                        auto_pull_on_connect: settings.auto_pull_on_connect,
                        skip_push_verification: enabled,
                    })
                    .map_err(|e| {
                        crate::error::AppError::new(
                            crate::error::ErrorKind::StorageError,
                            e.to_string(),
                        )
                    })
                },
                |result| Message::Editor(EditorMessage::SettingsSaved { result }),
            )
        }
        Message::Editor(EditorMessage::SettingsSaved { result }) => {
            if let Err(e) = result {
                window
                    .diagnostics
                    .push(crate::diagnostics::DiagnosticEvent::new(
                        crate::diagnostics::LogLevel::Error,
                        crate::diagnostics::Source::UI,
                        format!("Failed to save settings: {}", e),
                    ));
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
