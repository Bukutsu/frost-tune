use crate::models::{MAX_GLOBAL_GAIN, MIN_GLOBAL_GAIN};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_8, TYPE_BODY, TYPE_CAPTION};
use crate::ui::views::action_button;
use iced::widget::{column, container, pick_list, row, slider, text, text_input};
use iced::{Element, Length};
use iced::font::{Font, Weight};

pub fn view_tools_panel(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // --- AUTOEQ ACTIONS ---
    let autoeq_section = column![
        text("AUTO-EQ").size(TYPE_CAPTION).color(TOKYO_NIGHT_MUTED),
        row![
            action_button("Import File")
                .on_press_maybe(if is_busy { None } else { Some(Message::ImportFromFilePressed) })
                .style(theme::pill_primary_button)
                .width(Length::Fill),
            action_button("Import Clipboard")
                .on_press_maybe(if is_busy { None } else { Some(Message::ImportFromClipboard) })
                .style(theme::pill_primary_button)
                .width(Length::Fill),
        ].spacing(SPACE_8),
        row![
            action_button("Export File")
                .on_press_maybe(if is_busy { None } else { Some(Message::ExportToFilePressed) })
                .style(theme::pill_secondary_button)
                .width(Length::Fill),
            action_button("Export Clipboard")
                .on_press_maybe(if is_busy { None } else { Some(Message::ExportAutoEQPressed) })
                .style(theme::pill_secondary_button)
                .width(Length::Fill),
        ].spacing(SPACE_8),
    ].spacing(SPACE_8);

    // --- PRESETS ---
    let preset_names: Vec<String> = state
        .editor_state
        .profiles
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let select_preset = row![
        pick_list(
            preset_names,
            state.editor_state.selected_profile_name.clone(),
            move |p| {
                if is_busy {
                    Message::None
                } else {
                    Message::ProfileSelected(p)
                }
            },
        )
        .placeholder("Select Preset")
        .style(theme::m3_input_pick_list)
        .width(Length::Fill),
        action_button("⟳")
            .on_press_maybe(if is_busy { None } else { Some(Message::ReloadProfilesPressed) })
            .style(theme::pill_text_button)
            .width(Length::Fixed(32.0)),
        action_button("📁")
            .on_press_maybe(if is_busy { None } else { Some(Message::OpenProfilesDirPressed) })
            .style(theme::pill_text_button)
            .width(Length::Fixed(32.0)),
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    let profile_name_input = {
        let input = text_input("New Name...", &state.editor_state.new_profile_name)
            .style(theme::m3_filled_input);
        
        if is_busy {
            input
        } else {
            input.on_input(Message::ProfileNameInput)
        }
    };

    let actions_row = row![
        action_button("Reset")
            .on_press_maybe(if is_busy { None } else { Some(Message::ResetFiltersPressed) })
            .style(theme::pill_secondary_button),
        action_button("Save")
            .on_press_maybe(if is_busy { None } else { Some(Message::SaveProfilePressed) })
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

    let preset_section = column![
        text("PRESET").size(TYPE_CAPTION).color(TOKYO_NIGHT_MUTED),
        select_preset.width(Length::Fill),
        profile_name_input.width(Length::Fill),
        actions_row.width(Length::Fill),
    ]
    .spacing(SPACE_12);

    // --- PREAMP ---
    let preamp_section = column![
        text("PREAMP").size(TYPE_CAPTION).color(TOKYO_NIGHT_MUTED),
        row![
            slider(
                MIN_GLOBAL_GAIN as f64..=MAX_GLOBAL_GAIN as f64,
                state.editor_state.global_gain as f64,
                move |v| {
                    if is_busy {
                        Message::None
                    } else {
                        Message::GlobalGainChanged(v as i8)
                    }
                }
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
    .spacing(SPACE_8);

    container(column![autoeq_section, preset_section, preamp_section].spacing(SPACE_16))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
