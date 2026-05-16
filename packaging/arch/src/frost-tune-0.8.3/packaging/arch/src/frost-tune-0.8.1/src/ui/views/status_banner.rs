use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::MainWindow;
use crate::ui::theme::{
    self, TOKYO_NIGHT_BG, TOKYO_NIGHT_BLUE, TOKYO_NIGHT_GREEN, TOKYO_NIGHT_RED, TOKYO_NIGHT_YELLOW,
};
use crate::ui::tokens::{ICON_CLOSE, SPACE_16, TYPE_BODY};
use crate::ui::views::{icon_button, small_action_button};
use iced::widget::{column, container, row, text};
use iced::{Background, Border, Element, Length};

pub fn view_status_banner(state: &MainWindow) -> Element<'_, Message> {
    let banner_height = 36.0;

    if let Some(msg) = &state.editor_state.status_message {
        let color = match msg.severity {
            StatusSeverity::Info => TOKYO_NIGHT_BLUE,
            StatusSeverity::Success => TOKYO_NIGHT_GREEN,
            StatusSeverity::Warning => TOKYO_NIGHT_YELLOW,
            StatusSeverity::Error => TOKYO_NIGHT_RED,
        };
        let id = msg.id;

        let mut actions = row![].spacing(SPACE_16).align_y(iced::Alignment::Center);

        if matches!(
            msg.severity,
            StatusSeverity::Warning | StatusSeverity::Error
        ) {
            let btn = small_action_button("Details")
                .on_press(Message::ToggleDiagnostics)
                .style(theme::pill_text_button);
            actions = actions.push(btn);
        }

        actions = actions.push(
            icon_button(ICON_CLOSE)
                .on_press(Message::ClearStatusMessage(id))
                .style(theme::pill_text_button),
        );

        container(
            row![
                text(&msg.content).size(TYPE_BODY).color(TOKYO_NIGHT_BG),
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
                radius: 4.0.into(),
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
