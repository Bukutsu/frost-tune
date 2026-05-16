use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::models::*;
use crate::ui::main_window::parse_freq_string;
use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::{ConfirmAction, DraftFilter, MainWindow};
use iced::Task;

fn handle_band_text_input(
    window: &mut MainWindow,
    index: usize,
    s: String,
    setter: impl FnOnce(&mut DraftFilter, String),
) {
    let filter = match window.editor_state.data.filters.get(index) {
        Some(f) => f,
        None => {
            log::error!("Band input: index {} out of bounds", index);
            return;
        }
    };
    let draft = window
        .editor_state
        .session
        .input_buffer
        .active_draft
        .get_or_insert_with(|| DraftFilter::from_filter(filter));
    if draft.index != index {
        *draft = DraftFilter::from_filter(filter);
    }
    setter(draft, s);
}

fn cancel_band_draft_input(window: &mut MainWindow, index: usize) {
    if let Some(draft) = window.editor_state.session.input_buffer.active_draft.take() {
        if draft.index != index {
            window.editor_state.session.input_buffer.active_draft = Some(draft);
        }
    }
}

fn commit_band_field(
    window: &mut MainWindow,
    index: usize,
    parse_and_apply: impl FnOnce(&mut Filter, &mut DraftFilter) -> bool,
) -> Task<Message> {
    if let Some(mut draft) = window.editor_state.session.input_buffer.active_draft.take() {
        if draft.index == index {
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                if parse_and_apply(band, &mut draft) {
                    window.editor_state.session.is_dirty = true;
                } else {
                    window.editor_state.session.input_buffer.active_draft = Some(draft);
                }
            }
        } else {
            window.editor_state.session.input_buffer.active_draft = Some(draft);
        }
    }
    Task::none()
}

pub fn handle_editor(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::BandFreqChanged(index, freq) => {
            window.editor_state.push_undo();
            let freq_range = window.freq_range();
            let gain_range = window.gain_range();
            let q_range = window.q_range();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                band.freq = freq;
                band.enabled = true;
                band.clamp(freq_range, gain_range, q_range);
                window.editor_state.session.is_dirty = true;
            }
            Task::none()
        }
        Message::BandTypeChanged(index, t) => {
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                band.filter_type = t;
                window.editor_state.session.is_dirty = true;
            }
            Task::none()
        }
        Message::BandEnabledToggled(index, en) => {
            if !window.supports_per_band_enable() {
                return Task::none();
            }
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                band.enabled = en;
                window.editor_state.session.is_dirty = true;
            }
            Task::none()
        }
        Message::BandFreqInput(index, s) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.freq_input = val;
                draft.freq_error = None;
            });
            Task::none()
        }
        Message::BandGainInput(index, s) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.gain_input = val;
                draft.gain_error = None;
            });
            Task::none()
        }
        Message::BandQInput(index, s) => {
            handle_band_text_input(window, index, s, |draft, val| {
                draft.q_input = val;
                draft.q_error = None;
            });
            Task::none()
        }
        Message::BandFreqInputCommit(index) => {
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
        Message::BandGainInputCommit(index) => {
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
        Message::BandQInputCommit(index) => {
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
        Message::BandFreqInputCancel(index) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::BandGainInputCancel(index) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::BandQInputCancel(index) => {
            cancel_band_draft_input(window, index);
            Task::none()
        }
        Message::BandFreqSliderChanged(index, v) => {
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                let hz = 10f64.powf(v).round() as u16;
                band.freq = if window.editor_state.ui.snap_to_iso_enabled {
                    snap_freq_to_iso(hz)
                } else {
                    hz
                };
                window.editor_state.session.is_dirty = true;
                if let Some(draft) = window
                    .editor_state
                    .session
                    .input_buffer
                    .active_draft
                    .as_mut()
                {
                    if draft.index == index {
                        draft.freq_input = band.freq.to_string();
                    }
                }
            }
            Task::none()
        }
        Message::BandGainChanged(index, v) => {
            let (min_gain, max_gain) = window.gain_range();
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                band.gain = v.clamp(min_gain, max_gain);
                band.enabled = true;
                window.editor_state.session.is_dirty = true;
                if let Some(draft) = window
                    .editor_state
                    .session
                    .input_buffer
                    .active_draft
                    .as_mut()
                {
                    if draft.index == index {
                        draft.gain_input = format!("{:.2}", band.gain);
                    }
                }
            }
            Task::none()
        }
        Message::BandQChanged(index, v) => {
            window.editor_state.push_undo();
            if let Some(band) = window.editor_state.data.filters.get_mut(index) {
                let q_val = 10f64.powf(v);
                band.q = snap_q_to_iso(q_val);
                window.editor_state.session.is_dirty = true;
                if let Some(draft) = window
                    .editor_state
                    .session
                    .input_buffer
                    .active_draft
                    .as_mut()
                {
                    if draft.index == index {
                        draft.q_input = format!("{:.2}", band.q);
                    }
                }
            }
            Task::none()
        }
        Message::GlobalGainChanged(gain) => {
            window.editor_state.push_undo();
            window.editor_state.data.global_gain = gain.clamp(MIN_GLOBAL_GAIN, MAX_GLOBAL_GAIN);
            window.editor_state.session.is_dirty = true;
            Task::none()
        }
        Message::ResetFiltersPressed => {
            window.editor_state.session.pending_confirm = ConfirmAction::ResetFilters;
            Task::none()
        }
        Message::ConfirmResetFilters => {
            let num_bands = window.num_bands();
            if matches!(
                window.editor_state.session.pending_confirm,
                ConfirmAction::ResetFilters
            ) {
                window.editor_state.push_undo();
                window.editor_state.data.filters.clear();
                for i in 0..num_bands {
                    window
                        .editor_state
                        .data
                        .filters
                        .push(Filter::enabled(i as u8, false));
                }
                window.editor_state.data.global_gain = 0;
                window.editor_state.session.is_dirty = true;
                window.editor_state.session.is_autoeq_active = false;
                window.editor_state.session.input_buffer.active_draft = None;
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Reset filters to default",
                ));
                window.editor_state.session.pending_confirm = ConfirmAction::None;
                window.set_status("Filters reset to default", StatusSeverity::Info)
            } else {
                Task::none()
            }
        }
        Message::ToggleDiagnostics => {
            window.editor_state.ui.show_diagnostics = !window.editor_state.ui.show_diagnostics;
            Task::none()
        }

        Message::Undo => {
            if let Some(prev) = window.editor_state.session.undo_stack.pop() {
                let current = crate::models::PEQData {
                    filters: window.editor_state.data.filters.clone(),
                    global_gain: window.editor_state.data.global_gain,
                };
                window.editor_state.session.redo_stack.push(current);
                window.editor_state.data.filters = prev.filters;
                window.editor_state.data.global_gain = prev.global_gain;
                window.editor_state.session.is_dirty = true;
                window.editor_state.session.input_buffer.active_draft = None;
            }
            Task::none()
        }
        Message::Redo => {
            if let Some(next) = window.editor_state.session.redo_stack.pop() {
                let current = crate::models::PEQData {
                    filters: window.editor_state.data.filters.clone(),
                    global_gain: window.editor_state.data.global_gain,
                };
                window.editor_state.session.undo_stack.push(current);
                window.editor_state.data.filters = next.filters;
                window.editor_state.data.global_gain = next.global_gain;
                window.editor_state.session.is_dirty = true;
                window.editor_state.session.input_buffer.active_draft = None;
            }
            Task::none()
        }

        Message::ToggleSnapToIso(enabled) => {
            window.editor_state.ui.snap_to_iso_enabled = enabled;
            Task::none()
        }
        _ => Task::none(),
    }
}
