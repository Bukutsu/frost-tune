// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::*;
use crate::ui::state::AppState;
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_ERROR, COLOR_INFO, COLOR_ON_PRIMARY, COLOR_SUCCESS, COLOR_WARNING, ICON_CHECK_CIRCLE,
    ICON_CLOSE, ICON_ERROR, ICON_FONT, ICON_INFO, ICON_SIZE_SMALL, ICON_WARNING, SPACE_16, SPACE_4,
    TYPE_BODY,
};
use crate::ui::views::{icon_button, small_action_button};
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn view_status_banner(state: &AppState) -> Element<'_, Message> {
    if let Some(msg) = &state.editor.session.status_message {
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
                .on_press(Message::Diagnostics(DiagnosticsMessage::ToggleDiagnostics))
                .style(theme::m3_banner_text_button);
            actions = actions.push(btn);
        }

        actions = actions.push(
            icon_button(ICON_CLOSE)
                .on_press(Message::ClearStatusMessage(id))
                .style(theme::m3_banner_text_button),
        );

        container(
            row![
                text(icon)
                    .font(ICON_FONT)
                    .size(ICON_SIZE_SMALL)
                    .color(COLOR_ON_PRIMARY),
                text(&msg.content).size(TYPE_BODY).color(COLOR_ON_PRIMARY),
                container(text("")).width(Length::Fill),
                actions,
            ]
            .spacing(SPACE_16)
            .align_y(iced::Alignment::Center),
        )
        .padding([SPACE_4, SPACE_16])
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(move |theme| theme::status_banner_style(theme, color))
        .into()
    } else {
        container(column![])
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }
}
