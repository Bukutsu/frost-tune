use crate::diagnostics::{DiagnosticEvent, LogLevel};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{
    self, TOKYO_NIGHT_BLUE, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_RED, TOKYO_NIGHT_WARNING,
};
use crate::ui::tokens::{SPACE_12, SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_TINY};
use crate::ui::views::small_action_button;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

pub fn view_diagnostics(state: &MainWindow) -> Element<'_, Message> {
    let events: Vec<&DiagnosticEvent> = state
        .diagnostics
        .events()
        .filter(|e| !state.editor_state.diagnostics_errors_only || e.level == LogLevel::Error)
        .collect();

    let error_count = events.iter().filter(|e| e.level == LogLevel::Error).count();
    let warn_count = events.iter().filter(|e| e.level == LogLevel::Warn).count();
    let info_count = events.iter().filter(|e| e.level == LogLevel::Info).count();

    let summary = row![
        text("Errors:").size(TYPE_TINY).color(TOKYO_NIGHT_RED),
        text(format!("{}", error_count))
            .size(TYPE_TINY)
            .color(TOKYO_NIGHT_RED),
        text("Warnings:").size(TYPE_TINY).color(TOKYO_NIGHT_WARNING),
        text(format!("{}", warn_count))
            .size(TYPE_TINY)
            .color(TOKYO_NIGHT_WARNING),
        text("Info:").size(TYPE_TINY).color(TOKYO_NIGHT_BLUE),
        text(format!("{}", info_count))
            .size(TYPE_TINY)
            .color(TOKYO_NIGHT_BLUE),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center);

    let diag_events: Vec<Element<Message>> = events
        .into_iter()
        .rev()
        .take(30)
        .map(|e| {
            let c = match e.level {
                LogLevel::Info => TOKYO_NIGHT_MUTED,
                LogLevel::Warn => TOKYO_NIGHT_WARNING,
                LogLevel::Error => TOKYO_NIGHT_RED,
            };
            row![
                text(&e.timestamp)
                    .size(TYPE_TINY)
                    .color(TOKYO_NIGHT_MUTED)
                    .width(Length::Fixed(70.0)),
                text(format!("[{}]", e.source))
                    .size(TYPE_TINY)
                    .color(TOKYO_NIGHT_MUTED)
                    .width(Length::Fixed(40.0)),
                text(&e.message)
                    .size(TYPE_TINY)
                    .color(c)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_4)
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    let log_content = scrollable(column(diag_events).spacing(2)).height(Length::Shrink);

    container(
        column![
            row![
                text("DIAGNOSTICS")
                    .size(TYPE_CAPTION)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                container(text("")).width(Length::Fill),
                summary,
                small_action_button("Hide")
                    .on_press(Message::ToggleDiagnostics)
                    .style(theme::pill_text_button),
                small_action_button("Copy")
                    .on_press(Message::CopyDiagnostics)
                    .style(theme::pill_text_button),
                small_action_button("Clear")
                    .on_press(Message::ClearDiagnostics)
                    .style(theme::pill_text_button),
            ]
            .spacing(SPACE_8)
            .align_y(Alignment::Center),
            container(text("")).height(1.0).style(|_| container::Style {
                background: Some(iced::Background::Color(TOKYO_NIGHT_MUTED)),
                ..Default::default()
            }),
            log_content,
        ]
        .spacing(SPACE_4),
    )
    .padding(SPACE_12)
    .style(theme::card_style)
    .width(Length::Fill)
    .into()
}

pub fn view_diagnostics_section(state: &MainWindow) -> Element<'_, Message> {
    if state.editor_state.show_diagnostics {
        view_diagnostics(state)
    } else {
        container(
            row![
                text("DIAGNOSTICS")
                    .size(TYPE_CAPTION)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                container(text("")).width(Length::Fill),
                small_action_button("Show")
                    .on_press(Message::ToggleDiagnostics)
                    .style(theme::pill_text_button),
            ]
            .spacing(SPACE_8)
            .align_y(Alignment::Center),
        )
        .padding(SPACE_12)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
    }
}
