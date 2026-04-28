use crate::ui::messages::Message;
use crate::ui::theme::{self, TOKYO_NIGHT_FG, TOKYO_NIGHT_MUTED};
use crate::ui::tokens::{SPACE_12, SPACE_16, TYPE_LABEL, TYPE_TITLE};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn view_confirm_dialog<'a>(
    title: String,
    message: String,
    confirm_label: &'static str,
    confirm_msg: Message,
) -> Element<'a, Message> {
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
                    .style(theme::pill_danger_button),
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
