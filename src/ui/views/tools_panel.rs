// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::Message;
use crate::ui::state::{MainWindow, ToolsTab};
use crate::ui::theme;
use crate::ui::tokens::{
    BUTTON_HEIGHT_SMALL, COLOR_ON_SURFACE_VARIANT, ICON_BUTTON_SIZE, PROFILE_LIST_HEIGHT, SPACE_12,
    SPACE_16, SPACE_2, SPACE_8, TYPE_LABEL,
};
use crate::ui::views::{action_button, icon_action_button, icon_button};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

fn tab_button<'a>(
    label: &'a str,
    tab: ToolsTab,
    active: ToolsTab,
) -> iced::widget::Button<'a, Message> {
    let is_active = tab == active;
    button(
        container(
            text(label)
                .size(TYPE_LABEL)
                .align_x(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .padding(0.0)
    .height(Length::Fixed(BUTTON_HEIGHT_SMALL))
    .width(Length::Fill)
    .on_press(Message::ToolsTabSelected(tab))
    .style(move |t, s| theme::tab_button_style(t, s, is_active))
}

pub fn view_tools_panel(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // --- AUTOEQ ACTIONS ---
    let autoeq_section = column![
        row![
            icon_action_button(crate::ui::tokens::ICON_IMPORT_FILE, "Import File")
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::ImportFromFilePressed)
                })
                .style(theme::m3_outlined_button)
                .width(Length::Fill),
            icon_action_button(crate::ui::tokens::ICON_IMPORT_CLIPBOARD, "Paste")
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::ImportFromClipboard)
                })
                .style(theme::m3_outlined_button)
                .width(Length::Fill),
        ]
        .spacing(SPACE_8),
        row![
            icon_action_button(crate::ui::tokens::ICON_EXPORT_FILE, "Export File")
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::ExportToFilePressed)
                })
                .style(theme::m3_tonal_button)
                .width(Length::Fill),
            icon_action_button(crate::ui::tokens::ICON_EXPORT_CLIPBOARD, "Copy")
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::ExportAutoEQPressed)
                })
                .style(theme::m3_tonal_button)
                .width(Length::Fill),
        ]
        .spacing(SPACE_8),
    ]
    .spacing(SPACE_8);

    // --- PRESETS ---
    let search_query = state.editor_state.ui.profile_search.to_lowercase();
    let selected_name = state.editor_state.ui.selected_profile_name.as_deref();
    let filtered_profiles: Vec<&crate::storage::Profile> = state
        .editor_state
        .ui
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
        .ui
        .selected_profile_name
        .as_ref()
        .and_then(|name| {
            state
                .editor_state
                .ui
                .profiles
                .iter()
                .find(|p| &p.name == name)
                .and_then(|p| p.modified.as_deref())
        });

    let search_input = {
        let input = text_input("Search profiles…", &state.editor_state.ui.profile_search)
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
            .style(theme::m3_text_button)
            .width(Length::Fixed(ICON_BUTTON_SIZE)),
        icon_button(crate::ui::tokens::ICON_FOLDER)
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::OpenProfilesDirPressed)
            })
            .style(theme::m3_text_button)
            .width(Length::Fixed(ICON_BUTTON_SIZE)),
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    let profile_list: Element<'_, Message> = if filtered_profiles.is_empty() {
        container(
            text("No profiles found")
                .size(crate::ui::tokens::TYPE_CAPTION)
                .color(COLOR_ON_SURFACE_VARIANT),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .height(Length::Fixed(PROFILE_LIST_HEIGHT))
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(
                crate::ui::tokens::COLOR_SURFACE_DIM,
            )),
            ..Default::default()
        })
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
                .height(Length::Fixed(BUTTON_HEIGHT_SMALL))
                .width(Length::Fill)
                .style(move |theme, status| theme::profile_row_style(theme, status, is_selected))
                .on_press(Message::ProfileSelected(p.name.clone()))
                .into()
            })
            .collect();

        container(
            scrollable(column(rows).spacing(SPACE_2))
                .height(Length::Fixed(PROFILE_LIST_HEIGHT))
                .width(Length::Fill),
        )
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(
                crate::ui::tokens::COLOR_SURFACE_DIM,
            )),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding(crate::ui::tokens::SPACE_4)
        .into()
    };

    let profile_name_input = {
        let input = text_input("New Name…", &state.editor_state.session.new_profile_name)
            .style(theme::m3_filled_input);

        if is_busy {
            input
        } else {
            input.on_input(Message::ProfileNameInput)
        }
    };

    let undo_redo_row = row![
        action_button("Undo")
            .on_press_maybe(
                if is_busy || state.editor_state.session.undo_stack.is_empty() {
                    None
                } else {
                    Some(Message::Undo)
                }
            )
            .style(theme::m3_tonal_button),
        action_button("Redo")
            .on_press_maybe(
                if is_busy || state.editor_state.session.redo_stack.is_empty() {
                    None
                } else {
                    Some(Message::Redo)
                }
            )
            .style(theme::m3_tonal_button),
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    let actions_row = row![
        action_button("Reset")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ResetFiltersPressed)
            })
            .style(theme::m3_tonal_button),
        action_button("Save")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::SaveProfilePressed)
            })
            .style(theme::m3_filled_button),
        if !is_busy && state.editor_state.ui.selected_profile_name.is_some() {
            action_button("Delete")
                .on_press(Message::DeleteProfilePressed)
                .style(theme::m3_outlined_button_error)
        } else {
            action_button("Delete").style(theme::m3_outlined_button_error)
        },
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    let preset_body = column![
        profile_search_row.width(Length::Fill),
        profile_list,
        {
            let date_widget: Element<'_, Message> = if let Some(date) = selected_profile_modified {
                text(format!("Modified: {}", date))
                    .size(crate::ui::tokens::TYPE_CAPTION)
                    .color(COLOR_ON_SURFACE_VARIANT)
                    .into()
            } else {
                text("").into()
            };
            date_widget
        },
        checkbox(state.editor_state.ui.snap_to_iso_enabled)
            .label("Snap to ISO frequencies")
            .on_toggle(Message::ToggleSnapToIso)
            .size(16)
            .text_size(crate::ui::tokens::TYPE_CAPTION)
            .style(theme::checkbox_style),
        profile_name_input.width(Length::Fill),
    ]
    .spacing(SPACE_12);

    let active_tab = state.editor_state.ui.active_tools_tab;
    let tab_strip = row![
        tab_button("Preset", ToolsTab::Preset, active_tab),
        tab_button("AUTO-EQ", ToolsTab::AutoEq, active_tab),
    ]
    .spacing(SPACE_8);

    let tab_body: Element<'_, Message> = match active_tab {
        ToolsTab::Preset => preset_body.into(),
        ToolsTab::AutoEq => autoeq_section.into(),
    };

    let shared_actions = column![
        undo_redo_row.width(Length::Fill),
        actions_row.width(Length::Fill),
    ]
    .spacing(SPACE_12);

    container(column![tab_strip, tab_body, shared_actions].spacing(SPACE_16))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
