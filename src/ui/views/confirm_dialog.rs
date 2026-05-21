// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::Message;
use crate::ui::theme;
use crate::ui::tokens::{
    BUTTON_HEIGHT_LARGE, BUTTON_HORIZONTAL_PADDING, COLOR_ON_SURFACE, COLOR_ON_SURFACE_VARIANT,
    DIALOG_WIDTH, DIALOG_WIDTH_SMALL, SPACE_12, SPACE_16, SPACE_8, TYPE_LABEL, TYPE_TITLE,
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
pub fn view_import_dialog<'a>(
    title: String,
    message: String,
    input_value: &'a str,
    input_placeholder: &'a str,
    profiles: &'a [crate::storage::Profile],
    active_profile_name: Option<&'a str>,
    confirm_label: &'static str,
    confirm_msg: Message,
) -> Element<'a, Message> {
    let input = text_input(input_placeholder, input_value)
        .on_input(Message::ImportNameInput)
        .on_submit(confirm_msg.clone())
        .style(theme::m3_filled_input)
        .width(Length::Fill);

    let profile_names: Vec<String> = profiles.iter().map(|p| p.name.clone()).collect();
    let selected_name = profile_names
        .iter()
        .find(|&name| name == input_value)
        .cloned();

    let dropdown = pick_list(profile_names, selected_name, Message::ImportProfileSelected)
        .placeholder("Select profile to overwrite...")
        .style(theme::m3_input_pick_list)
        .width(Length::Fill);

    let mut inner_col = column![
        text("Profile Name:")
            .size(TYPE_LABEL)
            .color(COLOR_ON_SURFACE),
        input,
    ]
    .spacing(SPACE_8)
    .width(Length::Fill);

    if !profiles.is_empty() {
        inner_col = inner_col.push(
            column![
                text("Or Overwrite Existing:")
                    .size(TYPE_LABEL)
                    .color(COLOR_ON_SURFACE),
                dropdown,
            ]
            .spacing(SPACE_8),
        );
    }

    let mut quick_actions = column![].spacing(SPACE_8).width(Length::Fill);

    quick_actions = quick_actions.push(
        action_button("Apply directly to EQ (Unsaved)")
            .on_press(Message::ImportDirectlyToEditor)
            .style(theme::m3_outlined_button)
            .width(Length::Fill),
    );

    if let Some(active_name) = active_profile_name {
        quick_actions = quick_actions.push(
            button(
                container(
                    text(format!("Overwrite active '{}'", active_name))
                        .size(TYPE_LABEL)
                        .align_x(iced::Alignment::Center),
                )
                .height(Length::Fill)
                .center_y(Length::Fill),
            )
            .padding([0.0, BUTTON_HORIZONTAL_PADDING])
            .height(Length::Fixed(BUTTON_HEIGHT_LARGE))
            .on_press(Message::ImportOverwriteActive)
            .style(theme::m3_outlined_button_error)
            .width(Length::Fill),
        );
    }

    inner_col = inner_col.push(
        column![
            text("Quick Actions:")
                .size(TYPE_LABEL)
                .color(COLOR_ON_SURFACE),
            quick_actions,
        ]
        .spacing(SPACE_8),
    );

    let actions = row![
        action_button("Cancel")
            .on_press(Message::DismissConfirmDialog)
            .style(theme::m3_tonal_button)
            .width(Length::Fill),
        action_button(confirm_label)
            .on_press(confirm_msg)
            .style(theme::m3_filled_button)
            .width(Length::Fill),
    ]
    .spacing(SPACE_12)
    .width(Length::Fill);

    let col = column![
        text(title).size(TYPE_TITLE).color(COLOR_ON_SURFACE),
        text(message)
            .size(TYPE_LABEL)
            .color(COLOR_ON_SURFACE_VARIANT),
        inner_col,
        actions,
    ]
    .spacing(SPACE_16);

    container(col.padding(SPACE_16))
        .style(theme::dialog_style)
        .width(Length::Fixed(DIALOG_WIDTH))
        .into()
}
