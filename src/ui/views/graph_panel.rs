// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::graph::EqGraph;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::tokens::{ELEVATION_1, SPACE_12, SPACE_16};
use iced::widget::{canvas, container};
use iced::{Color, Element, Length, Padding};

/// Graph with a fixed height based on width — for scrollable layouts (narrow, medium).
pub fn view_graph(state: &MainWindow) -> Element<'_, Message> {
    iced::widget::responsive(move |size| {
        let height = if size.width < 600.0 {
            240.0
        } else if size.width < 1000.0 {
            280.0
        } else {
            320.0
        };

        container(
            canvas(EqGraph::new(
                &state.editor_state.data.filters,
                state.editor_state.data.global_gain,
                &state.editor_state.ui.graph_state,
            ))
            .width(Length::Fill)
            .height(Length::Fixed(height)),
        )
        .padding(Padding {
            top: SPACE_16,
            right: SPACE_12,
            bottom: SPACE_12,
            left: SPACE_12,
        })
        .style(graph_container_style)
        .width(Length::Fill)
        .into()
    })
    .into()
}

/// Graph that expands to fill all available vertical space — for the wide layout.
pub fn view_graph_fill(state: &MainWindow) -> Element<'_, Message> {
    container(
        canvas(EqGraph::new(
            &state.editor_state.data.filters,
            state.editor_state.data.global_gain,
            &state.editor_state.ui.graph_state,
        ))
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(Padding {
        top: SPACE_16,
        right: SPACE_12,
        bottom: SPACE_12,
        left: SPACE_12,
    })
    .style(graph_container_style)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn graph_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(ELEVATION_1.into()),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}
