use crate::diagnostics::LogLevel;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_ERROR, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_2, SPACE_8, TYPE_LABEL, TYPE_TITLE};
use crate::ui::views::action_button;
use iced::widget::{checkbox, column, container, row, scrollable, text};
use iced::{Element, Length};

pub fn view_diagnostics(state: &MainWindow) -> Element<'_, Message> {
    let has_events = state.diagnostics.count() > 0;
    let diag_events: Vec<Element<Message>> = state
        .diagnostics
        .events()
        .filter(|e| !state.editor_state.diagnostics_errors_only || e.level == LogLevel::Error)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take(20)
        .map(|e| {
            let c = match e.level {
                LogLevel::Info => TOKYO_NIGHT_MUTED,
                LogLevel::Warn => TOKYO_NIGHT_WARNING,
                LogLevel::Error => TOKYO_NIGHT_ERROR,
            };
            text(format!("[{}] {} {}", e.level, e.source, e.message))
                .size(TYPE_LABEL)
                .color(c)
                .into()
        })
        .collect();

    let visible_count = diag_events.len().min(20);
    let logs_height = if visible_count == 0 {
        84.0
    } else if visible_count < 4 {
        92.0
    } else if visible_count < 10 {
        120.0
    } else {
        160.0
    };

    container(
        column![
            row![
                checkbox(state.editor_state.diagnostics_errors_only)
                    .label("Errors Only")
                    .on_toggle(Message::ToggleDiagnosticsErrorsOnly)
                    .size(14),
                container(text("")).width(Length::Fill),
            ]
            .align_y(iced::Alignment::Center),
            if diag_events.is_empty() {
                container(
                    text("No diagnostics events yet. Events will appear during connection, import, and sync operations.")
                        .size(TYPE_LABEL)
                        .color(TOKYO_NIGHT_MUTED),
                )
                .height(Length::Fixed(logs_height))
                .align_y(iced::alignment::Vertical::Center)
            } else {
                container(scrollable(column(diag_events).spacing(SPACE_2)).height(Length::Fixed(logs_height)))
            },
            row![
                action_button("Copy")
                    .on_press(Message::CopyDiagnostics)
                    .style(theme::pill_text_button),
                action_button("Export")
                    .on_press(Message::ExportDiagnosticsToFile)
                    .style(theme::pill_text_button),
                action_button("Clear")
                    .on_press(Message::ClearDiagnostics)
                    .style(if has_events {
                        theme::pill_outlined_danger_button
                    } else {
                        theme::pill_text_button
                    }),
            ]
            .spacing(SPACE_8),
        ]
        .spacing(SPACE_12),
    )
    .padding(SPACE_16)
    .style(theme::card_style)
    .width(Length::FillPortion(1))
    .into()
}

pub fn view_diagnostics_section(state: &MainWindow) -> Element<'_, Message> {
    let expanded = state.editor_state.diagnostics_expanded;
    let toggle_text = if expanded {
        "Hide diagnostics"
    } else {
        "Show diagnostics"
    };

    let header = row![
        text("Diagnostics")
            .size(TYPE_TITLE)
            .color(TOKYO_NIGHT_PRIMARY),
        container(text("")).width(Length::Fill),
        action_button(toggle_text)
            .on_press(Message::ToggleDiagnosticsExpanded(!expanded))
            .style(theme::pill_secondary_button),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(SPACE_12);

    let body: Element<Message> = if expanded {
        view_diagnostics(state)
    } else {
        container(
            text("Diagnostics are hidden by default. Expand when troubleshooting.")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED),
        )
        .padding([SPACE_12, SPACE_8])
        .into()
    };

    container(column![header, body].spacing(SPACE_12))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::FillPortion(1))
        .into()
}
