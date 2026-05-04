use crate::models::{MAX_GLOBAL_GAIN, MIN_GLOBAL_GAIN};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_8, TYPE_CAPTION};
use crate::ui::views::{action_button, icon_action_button, icon_button};
use iced::widget::{column, container, pick_list, row, slider, text, text_input};
use iced::{Element, Length};

pub fn view_tools_panel(state: &MainWindow, show_preamp: bool) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // --- AUTOEQ ACTIONS ---
    let autoeq_section = column![
        text("AUTO-EQ")
            .size(TYPE_CAPTION)
            .color(theme::TOKYO_NIGHT_FG)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        icon_action_button(crate::ui::tokens::ICON_IMPORT_FILE, "Import File")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ImportFromFilePressed)
            })
            .style(theme::pill_outlined_primary_button)
            .width(Length::Fill),
        icon_action_button(crate::ui::tokens::ICON_IMPORT_CLIPBOARD, "Paste")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ImportFromClipboard)
            })
            .style(theme::pill_outlined_primary_button)
            .width(Length::Fill),
        icon_action_button(crate::ui::tokens::ICON_EXPORT_FILE, "Export File")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ExportToFilePressed)
            })
            .style(theme::pill_secondary_button)
            .width(Length::Fill),
        icon_action_button(crate::ui::tokens::ICON_EXPORT_CLIPBOARD, "Copy")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ExportAutoEQPressed)
            })
            .style(theme::pill_secondary_button)
            .width(Length::Fill),
    ]
    .spacing(SPACE_8);

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
        icon_button(crate::ui::tokens::ICON_RELOAD)
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ReloadProfilesPressed)
            })
            .style(theme::pill_text_button)
            .width(Length::Fixed(36.0)),
        icon_button(crate::ui::tokens::ICON_FOLDER)
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::OpenProfilesDirPressed)
            })
            .style(theme::pill_text_button)
            .width(Length::Fixed(36.0)),
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
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ResetFiltersPressed)
            })
            .style(theme::pill_secondary_button),
        action_button("Save")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::SaveProfilePressed)
            })
            .style(theme::pill_outlined_primary_button),
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
        text("PRESET")
            .size(TYPE_CAPTION)
            .color(theme::TOKYO_NIGHT_FG)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        select_preset.width(Length::Fill),
        profile_name_input.width(Length::Fill),
        actions_row.width(Length::Fill),
    ]
    .spacing(SPACE_12);

    let preamp_section = column![
        text(format!("PREAMP: {} dB", state.editor_state.global_gain))
            .size(TYPE_CAPTION)
            .color(theme::TOKYO_NIGHT_FG)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        row![slider(
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
        .width(Length::Fill)
        .style(theme::slider_style),]
        .spacing(SPACE_12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(SPACE_8);

    let sections = if show_preamp {
        column![preamp_section, autoeq_section, preset_section].spacing(SPACE_16)
    } else {
        column![autoeq_section, preset_section].spacing(SPACE_16)
    };

    container(sections)
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
