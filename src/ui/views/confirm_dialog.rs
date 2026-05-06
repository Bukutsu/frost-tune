use crate::ui::messages::Message;
use crate::ui::theme::{self, TOKYO_NIGHT_FG, TOKYO_NIGHT_MUTED};
use crate::ui::tokens::{SPACE_12, SPACE_16, TYPE_LABEL, TYPE_TITLE};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, text, text_input};
use iced::{Element, Length};

pub fn view_confirm_dialog<'a>(
    title: String,
    message: String,
    confirm_label: &'static str,
    confirm_msg: Message,
    is_danger: bool,
) -> Element<'a, Message> {
    let confirm_style = if is_danger {
        theme::pill_danger_button
    } else {
        theme::pill_primary_button
    };

    container(
        column![
            text(title).size(TYPE_TITLE).color(TOKYO_NIGHT_FG),
            text(message).size(TYPE_LABEL).color(TOKYO_NIGHT_MUTED),
            row![
                action_button("Cancel")
                    .on_press(Message::DismissConfirmDialog)
                    .style(theme::pill_secondary_button),
                action_button(confirm_label)
                    .on_press(confirm_msg)
                    .style(confirm_style),
            ]
            .spacing(SPACE_12),
        ]
        .spacing(SPACE_12)
        .padding(SPACE_16),
    )
    .style(theme::card_style)
    .width(Length::Fixed(400.0))
    .center_x(Length::Fill)
    .into()
}

pub fn view_name_input_dialog<'a>(
    title: String,
    message: String,
    input_value: &'a str,
    confirm_label: &'static str,
    confirm_msg: Message,
    is_danger: bool,
) -> Element<'a, Message> {
    let confirm_style = if is_danger {
        theme::pill_danger_button
    } else {
        theme::pill_primary_button
    };

    let input = text_input("Profile name...", input_value)
        .on_input(Message::ImportNameInput)
        .on_submit(confirm_msg.clone())
        .style(theme::m3_filled_input)
        .width(Length::Fill);

    container(
        column![
            text(title).size(TYPE_TITLE).color(TOKYO_NIGHT_FG),
            text(message).size(TYPE_LABEL).color(TOKYO_NIGHT_MUTED),
            input,
            row![
                action_button("Cancel")
                    .on_press(Message::DismissConfirmDialog)
                    .style(theme::pill_secondary_button),
                action_button(confirm_label)
                    .on_press(confirm_msg)
                    .style(confirm_style),
            ]
            .spacing(SPACE_12),
        ]
        .spacing(SPACE_12)
        .padding(SPACE_16),
    )
    .style(theme::card_style)
    .width(Length::Fixed(400.0))
    .center_x(Length::Fill)
    .into()
}
