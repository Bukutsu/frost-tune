use crate::diagnostics::{DiagnosticEvent, LogLevel};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_ERROR, COLOR_INFO, COLOR_ON_SURFACE_VARIANT, COLOR_WARNING, DIAGNOSTICS_LEVEL_WIDTH,
    DIAGNOSTICS_TIME_WIDTH, DIVIDER_HEIGHT, SPACE_12, SPACE_2, SPACE_4, SPACE_8, TYPE_TINY,
};
use crate::ui::views::small_action_button;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

pub fn view_diagnostics(state: &MainWindow) -> Element<'_, Message> {
    let events: Vec<&DiagnosticEvent> = state
        .diagnostics
        .events()
        .filter(|e| !state.editor_state.ui.diagnostics_errors_only || e.level == LogLevel::Error)
        .collect();

    let error_count = events.iter().filter(|e| e.level == LogLevel::Error).count();
    let warn_count = events.iter().filter(|e| e.level == LogLevel::Warn).count();
    let info_count = events.iter().filter(|e| e.level == LogLevel::Info).count();

    let summary = row![
        text("Errors:").size(TYPE_TINY).color(COLOR_ERROR),
        text(format!("{}", error_count))
            .size(TYPE_TINY)
            .color(COLOR_ERROR),
        text("Warnings:").size(TYPE_TINY).color(COLOR_WARNING),
        text(format!("{}", warn_count))
            .size(TYPE_TINY)
            .color(COLOR_WARNING),
        text("Info:").size(TYPE_TINY).color(COLOR_INFO),
        text(format!("{}", info_count))
            .size(TYPE_TINY)
            .color(COLOR_INFO),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center);

    let diag_events: Vec<Element<Message>> = events
        .into_iter()
        .rev()
        .take(30)
        .map(|e| {
            let c = match e.level {
                LogLevel::Info => COLOR_ON_SURFACE_VARIANT,
                LogLevel::Warn => COLOR_WARNING,
                LogLevel::Error => COLOR_ERROR,
            };
            let level_prefix = match e.level {
                LogLevel::Info => "[I]",
                LogLevel::Warn => "[W]",
                LogLevel::Error => "[E]",
            };
            row![
                text(&e.timestamp)
                    .size(TYPE_TINY)
                    .color(COLOR_ON_SURFACE_VARIANT)
                    .width(Length::Fixed(DIAGNOSTICS_LEVEL_WIDTH)),
                text(format!("[{}]", e.source))
                    .size(TYPE_TINY)
                    .color(COLOR_ON_SURFACE_VARIANT)
                    .width(Length::Fixed(DIAGNOSTICS_TIME_WIDTH)),
                text(level_prefix)
                    .size(TYPE_TINY)
                    .color(c)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
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

    let log_content = scrollable(column(diag_events).spacing(SPACE_2)).height(Length::Shrink);

    container(
        column![
            row![
                super::section_header("DIAGNOSTICS".to_string()),
                container(text("")).width(Length::Fill),
                summary,
                small_action_button("Hide")
                    .on_press(Message::ToggleDiagnostics)
                    .style(theme::m3_text_button),
                small_action_button("Copy")
                    .on_press(Message::CopyDiagnostics)
                    .style(theme::m3_text_button),
                small_action_button("Clear")
                    .on_press(Message::ClearDiagnostics)
                    .style(theme::m3_text_button),
            ]
            .spacing(SPACE_8)
            .align_y(Alignment::Center),
            container(text(""))
                .height(DIVIDER_HEIGHT)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(COLOR_ON_SURFACE_VARIANT)),
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
    if state.editor_state.ui.show_diagnostics {
        view_diagnostics(state)
    } else {
        container(
            row![
                super::section_header("DIAGNOSTICS".to_string()),
                container(text("")).width(Length::Fill),
                small_action_button("Show")
                    .on_press(Message::ToggleDiagnostics)
                    .style(theme::m3_text_button),
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
