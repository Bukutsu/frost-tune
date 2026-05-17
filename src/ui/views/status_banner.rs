use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_ERROR, COLOR_INFO, COLOR_ON_PRIMARY, COLOR_SUCCESS, COLOR_WARNING, ICON_CHECK_CIRCLE,
    ICON_CLOSE, ICON_ERROR, ICON_FONT, ICON_INFO, ICON_WARNING, SHAPE_EXTRA_SMALL, SPACE_16,
    TYPE_BODY,
};
use crate::ui::views::{icon_button, small_action_button};
use iced::widget::{column, container, row, text};
use iced::{Background, Border, Element, Length};

pub fn view_status_banner(state: &MainWindow) -> Element<'_, Message> {
    let banner_height = 36.0;

    if let Some(msg) = &state.editor_state.session.status_message {
        let (color, icon) = match msg.severity {
            StatusSeverity::Info => (COLOR_INFO, ICON_INFO),
            StatusSeverity::Success => (COLOR_SUCCESS, ICON_CHECK_CIRCLE),
            StatusSeverity::Warning => (COLOR_WARNING, ICON_WARNING),
            StatusSeverity::Error => (COLOR_ERROR, ICON_ERROR),
        };
        let id = msg.id;

        let mut actions = row![].spacing(SPACE_16).align_y(iced::Alignment::Center);

        if matches!(
            msg.severity,
            StatusSeverity::Warning | StatusSeverity::Error
        ) {
            let btn = small_action_button("Details")
                .on_press(Message::ToggleDiagnostics)
                .style(theme::m3_text_button);
            actions = actions.push(btn);
        }

        actions = actions.push(
            icon_button(ICON_CLOSE)
                .on_press(Message::ClearStatusMessage(id))
                .style(theme::m3_text_button),
        );

        container(
            row![
                text(icon)
                    .font(ICON_FONT)
                    .size(18.0)
                    .color(COLOR_ON_PRIMARY),
                text(&msg.content).size(TYPE_BODY).color(COLOR_ON_PRIMARY),
                container(text("")).width(Length::Fill),
                actions,
            ]
            .spacing(SPACE_16)
            .align_y(iced::Alignment::Center),
        )
        .padding([4.0, SPACE_16])
        .width(Length::Fill)
        .height(Length::Fixed(banner_height))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: SHAPE_EXTRA_SMALL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        container(column![])
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }
}
