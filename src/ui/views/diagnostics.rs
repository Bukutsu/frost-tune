use crate::diagnostics::LogLevel;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_RED, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_16, SPACE_8, TYPE_TINY};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

pub fn view_diagnostics(state: &MainWindow) -> Element<'_, Message> {
    let diag_events: Vec<Element<Message>> = state
        .diagnostics
        .events()
        .filter(|e| !state.editor_state.diagnostics_errors_only || e.level == LogLevel::Error)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take(30)
        .map(|e| {
            let c = match e.level {
                LogLevel::Info => TOKYO_NIGHT_MUTED,
                LogLevel::Warn => TOKYO_NIGHT_WARNING,
                LogLevel::Error => TOKYO_NIGHT_RED,
            };
            text(format!("[{}] {}", e.source, e.message))
                .size(TYPE_TINY)
                .color(c)
                .into()
        })
        .collect();

    container(
        column![
            text("DIAGNOSTICS").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
            container(scrollable(column(diag_events).spacing(2)).height(Length::Fixed(120.0))),
            row![
                action_button("Copy")
                    .on_press(Message::CopyDiagnostics)
                    .style(theme::pill_text_button),
                action_button("Clear")
                    .on_press(Message::ClearDiagnostics)
                    .style(theme::pill_text_button),
            ]
            .spacing(SPACE_8),
        ]
        .spacing(SPACE_8),
    )
    .padding(SPACE_16)
    .style(theme::card_style)
    .width(Length::Fill)
    .into()
}

pub fn view_diagnostics_section(state: &MainWindow) -> Element<'_, Message> {
    // In the new layout, we just show the diagnostics directly or simplified
    view_diagnostics(state)
}
