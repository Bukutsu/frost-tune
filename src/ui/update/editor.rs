use crate::ui::state::{MainWindow, ConfirmAction, InputBuffer};
use crate::ui::messages::{Message, StatusSeverity};
use crate::models::*;
use crate::diagnostics::{DiagnosticEvent, LogLevel, Source};
use crate::ui::main_window::parse_freq_string;
use iced::Task;

pub fn handle_editor(window: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::BandFreqChanged(index, freq) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.freq = freq;
                band.enabled = true;
                band.clamp();
            }
            Task::none()
        }
        Message::BandTypeChanged(index, t) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.filter_type = t;
            }
            Task::none()
        }
        Message::BandFreqInput(index, s) => {
            window.editor_state.input_buffer.editing_freq = Some((index, s));
            Task::none()
        }
        Message::BandGainInput(index, s) => {
            window.editor_state.input_buffer.editing_gain = Some((index, s));
            Task::none()
        }
        Message::BandQInput(index, s) => {
            window.editor_state.input_buffer.editing_q = Some((index, s));
            Task::none()
        }
        Message::BandFreqInputCommit(index) => {
            if let Some((i, s)) = window.editor_state.input_buffer.editing_freq.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Some(v) = parse_freq_string(&s) {
                            band.freq = v.clamp(MIN_FREQ, MAX_FREQ);
                            band.enabled = true;
                            window.editor_state.input_buffer.freq_error = None;
                        } else {
                            window.editor_state.input_buffer.freq_error =
                                Some((index, "Freq: 20-20000 Hz".to_string()));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::BandGainInputCommit(index) => {
            if let Some((i, s)) = window.editor_state.input_buffer.editing_gain.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Ok(v) = s.trim().parse::<f64>() {
                            if v >= MIN_BAND_GAIN && v <= MAX_BAND_GAIN {
                                band.gain = v;
                                band.enabled = true;
                                window.editor_state.input_buffer.gain_error = None;
                            } else {
                                window.editor_state.input_buffer.gain_error = Some((
                                    index,
                                    format!(
                                        "Gain: {:.0} to {:.0}",
                                        MIN_BAND_GAIN, MAX_BAND_GAIN
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
            if let Some((i, s)) = window.editor_state.input_buffer.editing_q.take() {
                if i == index {
                    if let Some(band) = window.editor_state.filters.get_mut(index) {
                        if let Ok(v) = s.trim().parse::<f64>() {
                            if v >= MIN_Q && v <= MAX_Q {
                                band.q = v;
                                band.enabled = true;
                                window.editor_state.input_buffer.q_error = None;
                            } else {
                                window.editor_state.input_buffer.q_error =
                                    Some((index, format!("Q: {:.1} to {:.1}", MIN_Q, MAX_Q)));
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
            }
            Task::none()
        }
        Message::BandGainChanged(index, v) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                band.gain = v.clamp(MIN_BAND_GAIN, MAX_BAND_GAIN);
                band.enabled = true;
            }
            Task::none()
        }
        Message::BandQChanged(index, v) => {
            if let Some(band) = window.editor_state.filters.get_mut(index) {
                let q_val = 10f64.powf(v);
                band.q = snap_q_to_iso(q_val);
            }
            Task::none()
        }
        Message::GlobalGainChanged(gain) => {
            window.editor_state.global_gain = gain;
            Task::none()
        }
        Message::ResetFiltersPressed => {
            window.editor_state.pending_confirm = ConfirmAction::ResetFilters;
            Task::none()
        }
        Message::ConfirmResetFilters => {
            if matches!(
                window.editor_state.pending_confirm,
                ConfirmAction::ResetFilters
            ) {
                let default_filters: Vec<Filter> =
                    (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
                window.editor_state.filters = default_filters;
                window.editor_state.global_gain = 0;
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
        _ => Task::none(),
    }
}
