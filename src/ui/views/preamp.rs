// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{PREAMP_LABEL_WIDTH, SPACE_12};
use iced::widget::{container, row, slider};
use iced::{Element, Length};

pub fn view_preamp(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;
    let gain_range = state.global_gain_range();

    let preamp_section = row![
        container(super::section_header(format!(
            "PREAMP: {} dB",
            state.editor_state.data.global_gain
        )))
        .width(Length::Fixed(PREAMP_LABEL_WIDTH)),
        slider(
            *gain_range.start() as f64..=*gain_range.end() as f64,
            state.editor_state.data.global_gain as f64,
            move |v| {
                if is_busy {
                    Message::None
                } else {
                    Message::GlobalGainChanged(v as i8)
                }
            }
        )
        .width(Length::Fill)
        .style(theme::slider_style),
    ]
    .spacing(SPACE_12)
    .align_y(iced::Alignment::Center);

    container(preamp_section)
        .padding(SPACE_12)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
