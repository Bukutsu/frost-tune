// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::Message;
use crate::ui::theme;
use crate::ui::tokens::{
    BUTTON_HEIGHT_SMALL, COLOR_ON_SURFACE, COLOR_ON_SURFACE_VARIANT, DIALOG_WIDTH,
    DIALOG_WIDTH_SMALL, SPACE_0, SPACE_12, SPACE_16, SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_LABEL,
    TYPE_TITLE,
};
use crate::ui::views::action_button;
use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Length};

fn dialog_container<'a>(
    title: String,
    message: String,
    confirm_label: &'static str,
    confirm_msg: Message,
    is_danger: bool,
    extra_content: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let confirm_style = if is_danger {
        theme::m3_filled_button_error
    } else {
        theme::m3_filled_button
    };

    let mut col = column![
        text(title).size(TYPE_TITLE).color(COLOR_ON_SURFACE),
        text(message)
            .size(TYPE_LABEL)
            .color(COLOR_ON_SURFACE_VARIANT),
    ];

    if let Some(content) = extra_content {
        col = col.push(content);
    }

    col = col.push(
        row![
            action_button("Cancel")
                .on_press(Message::DismissConfirmDialog)
                .style(theme::m3_tonal_button),
            action_button(confirm_label)
                .on_press(confirm_msg)
                .style(confirm_style),
        ]
        .spacing(SPACE_12),
    );

    container(col.spacing(SPACE_12).padding(SPACE_16))
        .style(theme::dialog_style)
        .width(Length::Fixed(DIALOG_WIDTH_SMALL))
        .into()
}

pub fn view_confirm_dialog<'a>(
    title: String,
    message: String,
    confirm_label: &'static str,
    confirm_msg: Message,
    is_danger: bool,
) -> Element<'a, Message> {
    dialog_container(title, message, confirm_label, confirm_msg, is_danger, None)
}

pub fn view_exit_dialog<'a>(
    title: String,
    message: String,
    save_label: &'static str,
    save_msg: Message,
    exit_label: &'static str,
    exit_msg: Message,
) -> Element<'a, Message> {
    container(
        column![
            text(title).size(TYPE_TITLE).color(COLOR_ON_SURFACE),
            text(message)
                .size(TYPE_LABEL)
                .color(COLOR_ON_SURFACE_VARIANT),
            row![
                action_button("Cancel")
                    .on_press(Message::DismissConfirmDialog)
                    .style(theme::m3_tonal_button),
                action_button(exit_label)
                    .on_press(exit_msg)
                    .style(theme::m3_outlined_button_error),
                action_button(save_label)
                    .on_press(save_msg)
                    .style(theme::m3_filled_button),
            ]
            .spacing(SPACE_12),
        ]
        .spacing(SPACE_12)
        .padding(SPACE_16),
    )
    .style(theme::dialog_style)
    .width(Length::Fixed(DIALOG_WIDTH))
    .into()
}

pub fn view_name_input_dialog<'a>(
    title: String,
    message: String,
    input_value: &'a str,
    input_placeholder: &'a str,
    confirm_label: &'static str,
    confirm_msg: Message,
    is_danger: bool,
) -> Element<'a, Message> {
    let input = text_input(input_placeholder, input_value)
        .on_input(Message::ImportNameInput)
        .on_submit(confirm_msg.clone())
        .style(theme::m3_filled_input)
        .width(Length::Fill);

    dialog_container(
        title,
        message,
        confirm_label,
        confirm_msg,
        is_danger,
        Some(input.into()),
    )
}

#[allow(clippy::too_many_arguments)]
fn dialog_tab_button<'a>(label: &'a str, is_active: bool, msg: Message) -> Element<'a, Message> {
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
    .padding(SPACE_0)
    .height(Length::Fixed(BUTTON_HEIGHT_SMALL))
    .width(Length::Fill)
    .on_press(msg)
    .style(move |t, s| theme::tab_button_style(t, s, is_active))
    .into()
}

#[allow(clippy::too_many_arguments)]
pub fn view_import_dialog<'a>(
    title: String,
    message: String,
    input_value: &'a str,
    input_placeholder: &'a str,
    profiles: &'a [crate::storage::Profile],
    _active_profile_name: Option<&'a str>,
    import_temporary: bool,
    confirm_label: &'static str,
    confirm_msg: Message,
) -> Element<'a, Message> {
    let tab_strip = row![
        dialog_tab_button(
            "Try EQ (Temporary)",
            import_temporary,
            Message::ImportTemporaryToggled(true)
        ),
        dialog_tab_button(
            "Save to Profile",
            !import_temporary,
            Message::ImportTemporaryToggled(false)
        ),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    let mode_content: Element<'a, Message> = if import_temporary {
        container(
            column![
                text("Directly apply the preset filters to your active EQ bands.")
                    .size(TYPE_LABEL)
                    .color(COLOR_ON_SURFACE),
                text("This mode is temporary. Changes will not be saved to any profile file until you manually do so.")
                    .size(TYPE_LABEL)
                    .color(COLOR_ON_SURFACE_VARIANT),
            ]
            .spacing(SPACE_8)
        )
        .padding(SPACE_12)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
    } else {
        let input = text_input(input_placeholder, input_value)
            .on_input(Message::ImportNameInput)
            .on_submit(confirm_msg.clone())
            .style(theme::m3_filled_input)
            .width(Length::Fill);

        let inner_fields = if !profiles.is_empty() {
            let profile_names: Vec<String> = profiles.iter().map(|p| p.name.clone()).collect();
            let selected_name = profile_names
                .iter()
                .find(|&name| name == input_value)
                .cloned();

            let dropdown = pick_list(profile_names, selected_name, Message::ImportProfileSelected)
                .placeholder("Choose existing profile to overwrite...")
                .style(theme::m3_input_pick_list)
                .width(Length::Fill);

            column![
                column![
                    text("Profile Name")
                        .size(TYPE_LABEL)
                        .color(COLOR_ON_SURFACE),
                    input,
                ]
                .spacing(SPACE_8),
                column![
                    text("Or Overwrite Existing:")
                        .size(TYPE_CAPTION)
                        .color(COLOR_ON_SURFACE_VARIANT),
                    dropdown,
                ]
                .spacing(SPACE_4),
            ]
            .spacing(SPACE_12)
        } else {
            column![
                text("Profile Name")
                    .size(TYPE_LABEL)
                    .color(COLOR_ON_SURFACE),
                input,
            ]
            .spacing(SPACE_8)
        };

        container(inner_fields)
            .padding(SPACE_12)
            .style(theme::card_style)
            .width(Length::Fill)
            .into()
    };

    let confirm_btn = if import_temporary {
        action_button("Apply to EQ")
            .on_press(Message::ImportDirectlyToEditor)
            .style(theme::m3_filled_button)
            .width(Length::Fill)
    } else {
        action_button(confirm_label)
            .on_press(confirm_msg)
            .style(theme::m3_filled_button)
            .width(Length::Fill)
    };

    let actions = row![
        action_button("Cancel")
            .on_press(Message::DismissConfirmDialog)
            .style(theme::m3_tonal_button)
            .width(Length::Fill),
        confirm_btn,
    ]
    .spacing(SPACE_12)
    .width(Length::Fill);

    let col = column![
        text(title).size(TYPE_TITLE).color(COLOR_ON_SURFACE),
        text(message)
            .size(TYPE_LABEL)
            .color(COLOR_ON_SURFACE_VARIANT),
        tab_strip,
        mode_content,
        actions,
    ]
    .spacing(SPACE_16);

    container(col.padding(SPACE_16))
        .style(theme::dialog_style)
        .width(Length::Fixed(DIALOG_WIDTH))
        .into()
}
