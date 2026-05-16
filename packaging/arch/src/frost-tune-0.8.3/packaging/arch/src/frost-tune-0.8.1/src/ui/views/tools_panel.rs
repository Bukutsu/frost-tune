use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_2, SPACE_8};
use crate::ui::views::{action_button, icon_action_button, icon_button};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn view_tools_panel(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // --- AUTOEQ ACTIONS ---
    let autoeq_section = column![
        super::section_header("AUTO-EQ".to_string()),
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
    let search_query = state.editor_state.profile_search.to_lowercase();
    let selected_name = state.editor_state.selected_profile_name.as_deref();
    let filtered_profiles: Vec<&crate::storage::Profile> = state
        .editor_state
        .profiles
        .iter()
        .filter(|p| {
            search_query.is_empty()
                || p.name.to_lowercase().contains(&search_query)
                || selected_name == Some(&p.name)
        })
        .collect();

    let selected_profile_modified = state
        .editor_state
        .selected_profile_name
        .as_ref()
        .and_then(|name| {
            state
                .editor_state
                .profiles
                .iter()
                .find(|p| &p.name == name)
                .and_then(|p| p.modified.as_deref())
        });

    let search_input = {
        let input = text_input("Search profiles...", &state.editor_state.profile_search)
            .style(theme::m3_filled_input);
        if is_busy {
            input
        } else {
            input.on_input(Message::ProfileSearchInput)
        }
    };

    let profile_search_row = row![
        search_input.width(Length::Fill),
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

    let profile_list: Element<'_, Message> = if filtered_profiles.is_empty() {
        container(
            text("No profiles found")
                .size(crate::ui::tokens::TYPE_CAPTION)
                .color(crate::ui::theme::TOKYO_NIGHT_MUTED),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .height(Length::Fixed(36.0))
        .into()
    } else {
        let rows: Vec<Element<'_, Message>> = filtered_profiles
            .iter()
            .map(|p| {
                let is_selected = selected_name == Some(&p.name);
                button(
                    container(
                        text(&p.name)
                            .size(crate::ui::tokens::TYPE_BODY)
                            .align_y(iced::Alignment::Center),
                    )
                    .padding([0.0, SPACE_12])
                    .center_y(Length::Fill)
                    .height(Length::Fill),
                )
                .height(Length::Fixed(36.0))
                .width(Length::Fill)
                .style(move |theme, status| theme::profile_row_style(theme, status, is_selected))
                .on_press(Message::ProfileSelected(p.name.clone()))
                .into()
            })
            .collect();

        scrollable(column(rows).spacing(SPACE_2))
            .height(Length::Fixed(200.0))
            .width(Length::Fill)
            .into()
    };

    let profile_name_input = {
        let input = text_input("New Name...", &state.editor_state.new_profile_name)
            .style(theme::m3_filled_input);

        if is_busy {
            input
        } else {
            input.on_input(Message::ProfileNameInput)
        }
    };



    let undo_redo_row = row![
        action_button("Undo")
            .on_press_maybe(if is_busy || state.editor_state.undo_stack.is_empty() {
                None
            } else {
                Some(Message::Undo)
            })
            .style(theme::pill_secondary_button),
        action_button("Redo")
            .on_press_maybe(if is_busy || state.editor_state.redo_stack.is_empty() {
                None
            } else {
                Some(Message::Redo)
            })
            .style(theme::pill_secondary_button),
    ]
    .spacing(SPACE_8);
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
        super::section_header("PRESET".to_string()),
        profile_search_row.width(Length::Fill),
        profile_list,
        {
            let date_widget: Element<'_, Message> = if let Some(date) = selected_profile_modified {
                text(format!("Modified: {}", date))
                    .size(crate::ui::tokens::TYPE_CAPTION)
                    .color(crate::ui::theme::TOKYO_NIGHT_MUTED)
                    .into()
            } else {
                text("").into()
            };
            date_widget
        },
        row![
            checkbox(state.editor_state.snap_to_iso_enabled)
                .on_toggle(Message::ToggleSnapToIso)
                .style(theme::checkbox_style),
            text("Snap to ISO frequencies")
                .size(crate::ui::tokens::TYPE_CAPTION)
                .color(crate::ui::theme::TOKYO_NIGHT_FG),
        ]
        .spacing(SPACE_8)
        .align_y(iced::Alignment::Center),

        profile_name_input.width(Length::Fill),
        undo_redo_row.width(Length::Fill),
        actions_row.width(Length::Fill),
    ]
    .spacing(SPACE_12);
    container(column![autoeq_section, preset_section].spacing(SPACE_16))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
