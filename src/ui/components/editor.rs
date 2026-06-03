use crate::core::Filter;
use crate::hardware::OperationResult;
use iced::widget::canvas::Cache;

#[derive(Debug, Clone)]
pub enum EditorMessage {
    PullPressed,
    ConfirmPullPressed,
    PushPressed,
    ConfirmPushPressed,
    WorkerPulled(OperationResult),
    WorkerPushed(OperationResult),
    BandGainChanged(usize, f64),
    BandFreqChanged(usize, u16),
    BandQChanged(usize, f64),
    BandTypeChanged(usize, crate::core::FilterType),
    BandEnabledToggled(usize, bool),
    BandGainInput(usize, String),
    BandFreqInput(usize, String),
    BandQInput(usize, String),
    BandFreqSliderChanged(usize, f64),
    BandFreqSliderReleased(usize),
    BandGainReleased(usize),
    BandFreqInputCommit(usize),
    BandGainInputCommit(usize),
    BandQInputCommit(usize),
    BandFreqInputCancel(usize),
    BandGainInputCancel(usize),
    BandQInputCancel(usize),
    GlobalGainChanged(i8),
    ResetFiltersPressed,
    ConfirmResetFilters,
    ForceResetPressed,
    ConfirmForceResetPressed,
    Undo,
    Redo,
    ToggleSnapToIso(bool),
    ToggleAutoPullOnConnect(bool),
    ToggleSkipPushVerification(bool),
    SettingsSaved {
        result: Result<(), crate::error::AppError>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct InputBuffer {
    pub active_draft: Option<DraftFilter>,
}

impl InputBuffer {
    fn get_input_for(
        &self,
        band_index: usize,
        f: impl FnOnce(&DraftFilter) -> &str,
    ) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .map(f)
    }

    fn get_error_for(
        &self,
        band_index: usize,
        f: impl FnOnce(&DraftFilter) -> &Option<String>,
    ) -> Option<&str> {
        self.active_draft
            .as_ref()
            .filter(|d| d.index == band_index)
            .and_then(|d| f(d).as_deref())
    }

    pub fn get_freq_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.freq_input.as_str())
    }

    pub fn get_gain_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.gain_input.as_str())
    }

    pub fn get_q_input(&self, band_index: usize) -> Option<&str> {
        self.get_input_for(band_index, |d| d.q_input.as_str())
    }

    pub fn get_freq_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.freq_error)
    }

    pub fn get_gain_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.gain_error)
    }

    pub fn get_q_error(&self, band_index: usize) -> Option<&str> {
        self.get_error_for(band_index, |d| &d.q_error)
    }

    pub fn has_errors(&self) -> bool {
        self.active_draft.as_ref().is_some_and(|d| d.has_errors())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DraftFilter {
    pub index: usize,
    pub freq_input: String,
    pub gain_input: String,
    pub q_input: String,
    pub freq_error: Option<String>,
    pub gain_error: Option<String>,
    pub q_error: Option<String>,
}

impl DraftFilter {
    pub fn from_filter(filter: &Filter) -> Self {
        Self {
            index: filter.index as usize,
            freq_input: filter.freq.to_string(),
            gain_input: format!("{:.2}", filter.gain), // Format to 2 decimal places
            q_input: format!("{:.2}", filter.q),       // Format to 2 decimal places
            freq_error: None,
            gain_error: None,
            q_error: None,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.freq_error.is_some() || self.gain_error.is_some() || self.q_error.is_some()
    }
}

pub use crate::ui::messages::{EqSource, ToolsTab};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfirmAction {
    #[default]
    None,
    ResetFilters,
    DeleteProfile,
    ImportAutoEQ {
        data: std::sync::Arc<crate::core::PEQData>,
        default_name: String,
    },
    OverwriteProfile {
        name: String,
        data: std::sync::Arc<crate::core::PEQData>,
    },
    PullDevice,
    PushToDevice,
    ForceReset,
    LoadProfile {
        name: String,
    },
    ExitWithUnsavedChanges(iced::window::Id),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorData {
    pub peq: std::sync::Arc<crate::core::PEQData>,
    pub generation: u64,
}

pub const MAX_UNDO: usize = 50;

impl EditorComponent {
    pub fn push_undo(&mut self) {
        self.session.undo_stack.push(self.data.peq.clone());
        if self.session.undo_stack.len() > MAX_UNDO {
            self.session.undo_stack.remove(0);
        }
        self.session.redo_stack.clear();
        self.data.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PEQData;

    #[test]
    fn push_undo_clears_redo_and_adds_undo_entry() {
        let mut state = EditorComponent::default();
        state.session.redo_stack.push(std::sync::Arc::new(PEQData {
            filters: vec![],
            global_gain: 0,
        }));
        assert_eq!(state.session.undo_stack.len(), 0);
        assert_eq!(state.session.redo_stack.len(), 1);

        state.push_undo();

        assert_eq!(state.session.undo_stack.len(), 1);
        assert_eq!(state.session.redo_stack.len(), 0);
    }

    #[test]
    fn push_undo_trims_to_max() {
        let mut state = EditorComponent::default();
        for _ in 0..MAX_UNDO + 5 {
            state.push_undo();
        }
        assert_eq!(state.session.undo_stack.len(), MAX_UNDO);
    }
}

#[derive(Debug, Default)]
pub struct GraphState {
    pub grid_cache: Cache,
    pub curve_cache: Cache,
    pub cached_filters_hash: u64,
    pub cached_combined_response: Vec<f64>,
    pub cached_band_responses: Vec<Vec<f64>>,
}
use crate::storage::Profile;
use crate::ui::messages::StatusMessage;

#[derive(Debug, Default)]
pub struct EditorSession {
    pub is_dirty: bool,
    pub is_autoeq_active: bool,
    pub input_buffer: InputBuffer,
    pub pending_confirm: ConfirmAction,
    pub undo_stack: Vec<std::sync::Arc<crate::core::PEQData>>,
    pub redo_stack: Vec<std::sync::Arc<crate::core::PEQData>>,
    pub status_message: Option<StatusMessage>,
    pub import_name_input: String,
    pub new_profile_name: String,
    pub import_temporary: bool,
}

#[derive(Debug, Default)]
pub struct EditorUI {
    pub profiles: Vec<Profile>,
    pub selected_profile_name: Option<String>,
    pub profiles_dir_mtime: Option<std::time::SystemTime>,
    pub profile_search: String,
    pub diagnostics_errors_only: bool,
    pub show_diagnostics: bool,
    pub snap_to_iso_enabled: bool,
    pub active_tools_tab: ToolsTab,
    pub graph_state: GraphState,
    pub auto_pull_on_connect: bool,
    pub skip_push_verification: bool,
    pub eq_source: EqSource,
}

#[derive(Debug, Default)]
pub struct EditorComponent {
    pub data: EditorData,
    pub session: EditorSession,
    pub ui: EditorUI,
}

impl EditorComponent {
    pub fn view_preamp(
        &self,
        _is_busy: bool,
        gain_range: std::ops::RangeInclusive<i8>,
    ) -> iced::Element<'_, EditorMessage> {
        use crate::ui::theme;
        use crate::ui::tokens::{PREAMP_LABEL_WIDTH, SPACE_12};
        use crate::ui::views::section_header;
        use iced::widget::{container, row, slider};
        use iced::Length;

        let preamp_section = row![
            container(section_header(format!(
                "PREAMP: {} dB",
                self.data.peq.global_gain
            )))
            .width(Length::Fixed(PREAMP_LABEL_WIDTH)),
            slider(
                *gain_range.start() as f64..=*gain_range.end() as f64,
                self.data.peq.global_gain as f64,
                move |v| EditorMessage::GlobalGainChanged(v as i8)
            )
            .width(Length::Fill)
            .style(theme::slider_style),
        ]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center);

        container(preamp_section)
            .padding(SPACE_12)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
    }
}
