// Copyright (c) 2026 Buktsu
// SPDX-License-Identifier: MIT

use crate::diagnostics::{DiagnosticEvent, LogLevel};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_ERROR, COLOR_INFO, COLOR_ON_SURFACE_VARIANT, COLOR_WARNING, DIAG_LOG_MAX_HEIGHT,
    DIAG_MAX_VISIBLE_EVENTS, DIAG_SOURCE_WIDTH, DIAG_TIMESTAMP_WIDTH, SPACE_1, SPACE_12, SPACE_2,
    SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_TINY,
};
use crate::ui::views::{icon_button, small_action_button};
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

    let err_c = if error_count > 0 {
        COLOR_ERROR
    } else {
        COLOR_ON_SURFACE_VARIANT
    };
    let warn_c = if warn_count > 0 {
        COLOR_WARNING
    } else {
        COLOR_ON_SURFACE_VARIANT
    };
    let info_c = if info_count > 0 {
        COLOR_INFO
    } else {
        COLOR_ON_SURFACE_VARIANT
    };

    let header_row = row![
        super::section_header("DIAGNOSTICS".to_string()),
        container(text("")).width(Length::Fill),
        if state.editor_state.ui.diagnostics_errors_only {
            icon_button(crate::ui::tokens::ICON_WARNING)
                .on_press(Message::ToggleDiagnosticsErrorsOnly(false))
                .style(theme::m3_text_button_active)
        } else {
            icon_button(crate::ui::tokens::ICON_WARNING)
                .on_press(Message::ToggleDiagnosticsErrorsOnly(true))
                .style(theme::m3_text_button)
        },
        small_action_button("Copy")
            .on_press(Message::CopyDiagnostics)
            .style(theme::m3_text_button),
        small_action_button("Clear")
            .on_press(Message::ClearDiagnostics)
            .style(theme::m3_text_button),
        small_action_button("Export")
            .on_press(Message::ExportDiagnosticsToFile)
            .style(theme::m3_text_button),
        small_action_button("Hide")
            .on_press(Message::ToggleDiagnostics)
            .style(theme::m3_text_button),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center);

    let summary_bar = row![
        text(format!("E:{}", error_count))
            .size(TYPE_TINY)
            .color(err_c)
            .font(iced::Font::MONOSPACE),
        text(format!("W:{}", warn_count))
            .size(TYPE_TINY)
            .color(warn_c)
            .font(iced::Font::MONOSPACE),
        text(format!("I:{}", info_count))
            .size(TYPE_TINY)
            .color(info_c)
            .font(iced::Font::MONOSPACE),
    ]
    .spacing(SPACE_8)
    .align_y(Alignment::Center);

    let log_content: Element<'_, Message> = if events.is_empty() {
        container(
            text("No diagnostic events")
                .size(TYPE_CAPTION)
                .color(COLOR_ON_SURFACE_VARIANT),
        )
        .width(Length::Fill)
        .height(Length::Fixed(DIAG_LOG_MAX_HEIGHT))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    } else {
        let diag_events: Vec<Element<Message>> = events
            .into_iter()
            .rev()
            .take(DIAG_MAX_VISIBLE_EVENTS)
            .enumerate()
            .map(|(i, e)| {
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
                let bg = if i % 2 == 0 {
                    crate::ui::tokens::ELEVATION_0
                } else {
                    crate::ui::tokens::ELEVATION_1
                };

                container(
                    row![
                        text(
                            e.timestamp
                                .split_whitespace()
                                .last()
                                .unwrap_or(&e.timestamp),
                        )
                        .size(TYPE_TINY)
                        .color(COLOR_ON_SURFACE_VARIANT)
                        .font(iced::Font::MONOSPACE)
                        .width(Length::Fixed(DIAG_TIMESTAMP_WIDTH)),
                        text(format!("[{}]", e.source))
                            .size(TYPE_TINY)
                            .color(COLOR_ON_SURFACE_VARIANT)
                            .font(iced::Font::MONOSPACE)
                            .width(Length::Fixed(DIAG_SOURCE_WIDTH)),
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
                            .font(iced::Font::MONOSPACE)
                            .width(Length::Fill),
                    ]
                    .spacing(SPACE_4)
                    .align_y(Alignment::Start),
                )
                .padding([SPACE_2, SPACE_4])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    ..Default::default()
                })
                .into()
            })
            .collect();

        scrollable(column(diag_events).spacing(SPACE_1))
            .spacing(SPACE_8)
            .height(Length::Fixed(DIAG_LOG_MAX_HEIGHT))
            .width(Length::Fill)
            .into()
    };

    container(
        column![
            header_row,
            summary_bar,
            container(iced::widget::rule::horizontal(SPACE_1).style(theme::divider_rule_style))
                .width(Length::Fill),
            log_content,
        ]
        .spacing(SPACE_8),
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
