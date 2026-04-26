use crate::ui::messages::{Message, StatusSeverity};
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_BG, TOKYO_NIGHT_BLUE, TOKYO_NIGHT_GREEN, TOKYO_NIGHT_RED, TOKYO_NIGHT_YELLOW};
use crate::ui::tokens::{SPACE_16, SPACE_8, TYPE_BODY};
use crate::ui::views::action_button;
use iced::widget::{container, row, text};
use iced::{Background, Border, Element, Length};

pub fn view_status_banner(state: &MainWindow) -> Element<'_, Message> {
    if let Some(msg) = &state.editor_state.status_message {
        let color = match msg.severity {
            StatusSeverity::Info => TOKYO_NIGHT_BLUE,
            StatusSeverity::Success => TOKYO_NIGHT_GREEN,
            StatusSeverity::Warning => TOKYO_NIGHT_YELLOW,
            StatusSeverity::Error => TOKYO_NIGHT_RED,
        };

        container(
            row![
                text(&msg.content).size(TYPE_BODY).color(TOKYO_NIGHT_BG),
                container(text("")).width(Length::Fill),
                action_button("×")
                    .on_press(Message::ClearStatusMessage)
                    .style(theme::pill_text_button)
            ]
            .spacing(SPACE_16)
            .align_y(iced::Alignment::Center),
        )
        .padding([SPACE_8, SPACE_16])
        .width(Length::Fill)
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
        iced::widget::column![].into()
    }
}
