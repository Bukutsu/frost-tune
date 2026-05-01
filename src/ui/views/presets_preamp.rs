use crate::models::{MAX_GLOBAL_GAIN, MIN_GLOBAL_GAIN};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_8, TYPE_BODY, TYPE_CAPTION};
use crate::ui::views::action_button;
use iced::widget::{column, container, pick_list, row, slider, text, text_input};
use iced::{Element, Font, Length};
use iced::font::Weight;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetsLayout {
    Narrow,
    Medium,
    Wide,
}

pub fn view_presets_and_preamp(state: &MainWindow, layout: PresetsLayout) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;
    let is_narrow = matches!(layout, PresetsLayout::Narrow);
    let is_medium = matches!(layout, PresetsLayout::Medium);

    let preset_names: Vec<String> = state
        .editor_state
        .profiles
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let preset_placeholder: &str = if is_narrow { "Select" } else { "Select Preset" };

    let select_preset = row![
        pick_list(
            preset_names,
            state.editor_state.selected_profile_name.clone(),
            Message::ProfileSelected,
        )
        .placeholder(preset_placeholder)
        .style(theme::m3_input_pick_list)
        .width(Length::Fill),
        action_button("⟳")
            .on_press(Message::ReloadProfilesPressed)
            .style(theme::pill_text_button)
            .width(Length::Fixed(32.0)),
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    let profile_name_input = text_input("New Name...", &state.editor_state.new_profile_name)
        .on_input(Message::ProfileNameInput)
        .style(theme::m3_filled_input);

    let actions_row = row![
        action_button("Reset")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ResetFiltersPressed)
            })
            .style(theme::pill_secondary_button),
        action_button("Save")
            .on_press(Message::SaveProfilePressed)
            .style(theme::pill_primary_button),
        if !is_busy && state.editor_state.selected_profile_name.is_some() {
            action_button("Delete")
                .on_press(Message::DeleteProfilePressed)
                .style(theme::pill_danger_button)
        } else {
            action_button("Delete").style(theme::pill_danger_button)
        },
    ]
    .spacing(SPACE_12)
    .align_y(iced::Alignment::Center);

    let preset_section: Element<'_, Message> = if is_narrow {
        column![
            text("PRESET").size(TYPE_CAPTION).color(TOKYO_NIGHT_MUTED),
            select_preset.width(Length::Fill),
            profile_name_input.width(Length::Fill),
            actions_row.width(Length::Fill),
        ]
        .spacing(SPACE_12)
        .into()
    } else if is_medium {
        column![
            row![
                text("Presets:").size(TYPE_BODY).color(TOKYO_NIGHT_MUTED),
                select_preset.width(Length::FillPortion(3)),
                profile_name_input.width(Length::FillPortion(2)),
            ]
            .spacing(SPACE_12)
            .align_y(iced::Alignment::Center),
            actions_row,
        ]
        .spacing(SPACE_12)
        .into()
    } else {
        row![
            text("Presets:")
                .size(TYPE_BODY)
                .color(TOKYO_NIGHT_MUTED)
                .width(Length::Fixed(72.0)),
            select_preset.width(Length::Fixed(240.0)),
            profile_name_input.width(Length::Fill),
            actions_row,
        ]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let preamp_section: Element<'_, Message> = if is_narrow {
        column![
            text("PREAMP").size(TYPE_CAPTION).color(TOKYO_NIGHT_MUTED),
            row![
                slider(
                    MIN_GLOBAL_GAIN as f64..=MAX_GLOBAL_GAIN as f64,
                    state.editor_state.global_gain as f64,
                    |v| Message::GlobalGainChanged(v as i8)
                )
                .width(Length::Fill),
                text(format!("{} dB", state.editor_state.global_gain))
                    .size(TYPE_BODY)
                    .width(Length::Fixed(64.0))
                    .color(theme::ACCENT_VIBRANT)
                    .font(Font { weight: Weight::Bold, ..Default::default() }),
            ]
            .spacing(SPACE_12)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(SPACE_8)
        .into()
    } else {
        row![
            text("Preamp:").size(TYPE_BODY).color(TOKYO_NIGHT_MUTED),
            slider(
                MIN_GLOBAL_GAIN as f64..=MAX_GLOBAL_GAIN as f64,
                state.editor_state.global_gain as f64,
                |v| Message::GlobalGainChanged(v as i8)
            )
            .width(Length::Fill),
            text(format!("{} dB", state.editor_state.global_gain))
                .size(TYPE_BODY)
                .width(Length::Fixed(56.0))
                .color(TOKYO_NIGHT_PRIMARY),
        ]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center)
        .into()
    };

    container(column![preset_section, preamp_section].spacing(SPACE_16))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
