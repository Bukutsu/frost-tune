pub mod bands;
pub mod confirm_dialog;
pub mod diagnostics;
pub mod graph_panel;
pub mod header;
pub mod preamp;
pub mod status_banner;
pub mod tools_panel;

use crate::ui::messages::Message;
use crate::ui::tokens::{BUTTON_HORIZONTAL_PADDING, SPACE_4, TYPE_LABEL};
use iced::widget::{button, container, row, text};
use iced::Length;

pub fn action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            text(label)
                .size(TYPE_LABEL)
                .align_x(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .center_y(Length::Fill),
    )
    .padding([0.0, BUTTON_HORIZONTAL_PADDING])
    .height(Length::Fixed(48.0))
}

pub fn small_action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            text(label)
                .size(TYPE_LABEL)
                .align_x(iced::Alignment::Center),
        )
        .padding([0.0, 8.0])
        .center_y(Length::Fill),
    )
    .padding(0.0)
    .height(Length::Fixed(32.0))
}

pub fn icon_button<'a>(icon: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(text(icon).font(crate::ui::tokens::ICON_FONT).size(20.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .style(crate::ui::theme::pill_text_button)
    .padding(0.0)
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
}

pub fn toolbar_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            text(label)
                .size(TYPE_LABEL)
                .align_x(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .center_y(Length::Fill),
    )
    .padding([0.0, BUTTON_HORIZONTAL_PADDING])
    .height(Length::Fixed(36.0))
}

pub fn icon_action_button<'a>(icon: &'a str, label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            row![
                text(icon).font(crate::ui::tokens::ICON_FONT).size(18.0),
                text(label).size(TYPE_LABEL),
            ]
            .spacing(SPACE_4)
            .align_y(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .center_y(Length::Fill)
        .padding([0.0, BUTTON_HORIZONTAL_PADDING]),
    )
    .padding(0.0)
    .height(Length::Fixed(36.0))
}
