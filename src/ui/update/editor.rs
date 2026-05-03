use crate::ui::state::{MainWindow, ConfirmAction, InputBuffer};
use crate::ui::messages::{Message, StatusSeverity};
use crate::models::*;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::ui::main_window::parse_freq_string;
use iced::Task;

pub fn handle_editor(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::BandFreqChanged(index, freq) => {
            let freq_range = window.freq_range();
            let gain_range = window.gain_range();
            let q_range = window.q_range();
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.freq = freq;
                band.enabled = true;
                band.clamp(freq_range, gain_range, q_range);
                window.editor_state.is_dirty = true;
            }
            Task::none()
        }
        Message::BandTypeChanged(index, t) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.filter_type = t;
                window.editor_state.is_dirty = true;
            }
            Task::none()
        }
        Message::BandFreqInput(index, s) => {
            window.editor_state.input_buffer.editing_freq = Some((index, s));
            window.editor_state.input_buffer.clear_error(index);
            Task::none()
        }
        Message::BandGainInput(index, s) => {
            window.editor_state.input_buffer.editing_gain = Some((index, s));
            window.editor_state.input_buffer.clear_error(index);
            Task::none()
        }
        Message::BandQInput(index, s) => {
            window.editor_state.input_buffer.editing_q = Some((index, s));
            window.editor_state.input_buffer.clear_error(index);
            Task::none()
        }
        Message::BandFreqInputCommit(index) => {
            let (min_freq, max_freq) = window.freq_range();
            if let Some((i, s)) = window.editor_state.input_buffer.editing_freq.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Some(v) = parse_freq_string(&s) {
                            band.freq = v.clamp(min_freq, max_freq);
                            band.enabled = true;
                            window.editor_state.input_buffer.freq_error = None;
                            window.editor_state.is_dirty = true;
                        } else {
                            window.editor_state.input_buffer.freq_error =
                                Some((index, format!("Freq: {}-{} Hz", min_freq, max_freq)));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::BandGainInputCommit(index) => {
            let (min_gain, max_gain) = window.gain_range();
            if let Some((i, s)) = window.editor_state.input_buffer.editing_gain.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Ok(v) = s.trim().parse::<f64>() {
                            if v >= min_gain && v <= max_gain {
                                band.gain = v;
                                band.enabled = true;
                                window.editor_state.input_buffer.gain_error = None;
                                window.editor_state.is_dirty = true;
                            } else {
                                window.editor_state.input_buffer.gain_error = Some((
                                    index,
                                    format!(
                                        "Gain: {:.0} to {:.0}",
                                        min_gain, max_gain
                                    ),
                                ));
                            }
                        } else {
                            window.editor_state.input_buffer.gain_error =
                                Some((index, "Gain: enter number".to_string()));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::BandQInputCommit(index) => {
            let (min_q, max_q) = window.q_range();
            if let Some((i, s)) = window.editor_state.input_buffer.editing_q.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Ok(v) = s.trim().parse::<f64>() {
                            if v >= min_q && v <= max_q {
                                band.q = v;
                                band.enabled = true;
                                window.editor_state.input_buffer.q_error = None;
                                window.editor_state.is_dirty = true;
                            } else {
                                window.editor_state.input_buffer.q_error =
                                    Some((index, format!("Q: {:.1} to {:.1}", min_q, max_q)));
                            }
                        } else {
                            window.editor_state.input_buffer.q_error =
                                Some((index, "Q: enter number".to_string()));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::BandFreqInputCancel(index) => {
            if let Some((i, _)) = window.editor_state.input_buffer.editing_freq.take() {
                if i == index {}
            }
            Task::none()
        }
        Message::BandGainInputCancel(index) => {
            if let Some((i, _)) = window.editor_state.input_buffer.editing_gain.take() {
                if i == index {}
            }
            Task::none()
        }
        Message::BandQInputCancel(index) => {
            if let Some((i, _)) = window.editor_state.input_buffer.editing_q.take() {
                if i == index {}
            }
            Task::none()
        }
        Message::BandFreqSliderChanged(index, v) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                let hz = 10f64.powf(v).round() as u16;
                band.freq = snap_freq_to_iso(hz);
                window.editor_state.is_dirty = true;
            }
            Task::none()
        }
        Message::BandGainChanged(index, v) => {
            let (min_gain, max_gain) = window.gain_range();
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.gain = v.clamp(min_gain, max_gain);
                band.enabled = true;
                window.editor_state.is_dirty = true;
            }
            Task::none()
        }
        Message::BandQChanged(index, v) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                let q_val = 10f64.powf(v);
                band.q = snap_q_to_iso(q_val);
                window.editor_state.is_dirty = true;
            }
            Task::none()
        }
        Message::GlobalGainChanged(gain) => {
            window.editor_state.global_gain = gain.clamp(MIN_GLOBAL_GAIN, MAX_GLOBAL_GAIN);
            window.editor_state.is_dirty = true;
            Task::none()
        }
        Message::ResetFiltersPressed => {
            window.editor_state.pending_confirm = ConfirmAction::ResetFilters;
            Task::none()
        }
        Message::ConfirmResetFilters => {
            let num_bands = window.num_bands();
            if matches!(
                window.editor_state.pending_confirm,
                ConfirmAction::ResetFilters
            ) {
                window.editor_state.filters.clear();
                for i in 0..num_bands {
                    window.editor_state.filters.push(Filter::enabled(i as u8, false));
                }
                window.editor_state.global_gain = 0;
                window.editor_state.is_dirty = true;
                window.editor_state.is_autoeq_active = false;
                window.editor_state.input_buffer = InputBuffer::default();
                window.diagnostics.push(DiagnosticEvent::new(
                    LogLevel::Info,
                    Source::UI,
                    "Reset filters to default",
                ));
                window.editor_state.pending_confirm = ConfirmAction::None;
                window.set_status("Filters reset to default", StatusSeverity::Info)
            } else {
                Task::none()
            }
        }
        Message::ToggleDiagnostics => {
            window.editor_state.show_diagnostics = !window.editor_state.show_diagnostics;
            Task::none()
        }
        _ => Task::none(),
    }
}
