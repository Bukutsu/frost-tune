pub mod bands;
pub mod confirm_dialog;
pub mod diagnostics;
pub mod graph_panel;
pub mod header;
pub mod preamp;
pub mod status_banner;
pub mod tools_panel;

use crate::ui::messages::Message;
use crate::ui::tokens::{
    BUTTON_HEIGHT_COMPACT, BUTTON_HEIGHT_LARGE, BUTTON_HEIGHT_SMALL, BUTTON_HORIZONTAL_PADDING,
    COLOR_ON_SURFACE, ICON_BUTTON_SIZE, ICON_SIZE_MEDIUM, ICON_SIZE_SMALL, SPACE_4, SPACE_8,
    TYPE_CAPTION, TYPE_LABEL,
};
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

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
    .height(Length::Fixed(BUTTON_HEIGHT_LARGE))
}

pub fn small_action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            text(label)
                .size(TYPE_LABEL)
                .align_x(iced::Alignment::Center),
        )
        .padding([0.0, SPACE_8])
        .center_y(Length::Fill),
    )
    .padding(0.0)
    .height(Length::Fixed(BUTTON_HEIGHT_COMPACT))
}

pub fn icon_button<'a>(icon: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            text(icon)
                .font(crate::ui::tokens::ICON_FONT)
                .size(ICON_SIZE_MEDIUM),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .style(crate::ui::theme::m3_text_button)
    .padding(0.0)
    .width(Length::Fixed(ICON_BUTTON_SIZE))
    .height(Length::Fixed(ICON_BUTTON_SIZE))
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
    .height(Length::Fixed(BUTTON_HEIGHT_SMALL))
}

pub fn section_header<'a>(label: String) -> Element<'a, Message> {
    text(label)
        .size(TYPE_CAPTION)
        .color(COLOR_ON_SURFACE)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .into()
}

pub fn icon_action_button<'a>(icon: &'a str, label: &'a str) -> iced::widget::Button<'a, Message> {
    button(
        container(
            row![
                text(icon)
                    .font(crate::ui::tokens::ICON_FONT)
                    .size(ICON_SIZE_SMALL),
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
    .height(Length::Fixed(BUTTON_HEIGHT_SMALL))
}
