// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::graph::EqGraph;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::tokens::{
    ELEVATION_1, LAYOUT_GRAPH_BREAKPOINT_LARGE, LAYOUT_GRAPH_BREAKPOINT_SMALL,
    LAYOUT_GRAPH_HEIGHT_LARGE, LAYOUT_GRAPH_HEIGHT_MEDIUM, LAYOUT_GRAPH_HEIGHT_SMALL, SHAPE_NONE,
    SPACE_12, SPACE_16,
};
use iced::widget::{canvas, container};
use iced::{Color, Element, Length, Padding};

/// Graph with a fixed height based on width — for scrollable layouts (narrow, medium).
pub fn view_graph(state: &MainWindow) -> Element<'_, Message> {
    iced::widget::responsive(move |size| {
        let height = if size.width < LAYOUT_GRAPH_BREAKPOINT_SMALL {
            LAYOUT_GRAPH_HEIGHT_SMALL
        } else if size.width < LAYOUT_GRAPH_BREAKPOINT_LARGE {
            LAYOUT_GRAPH_HEIGHT_MEDIUM
        } else {
            LAYOUT_GRAPH_HEIGHT_LARGE
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
            radius: SHAPE_NONE.into(),
        },
        ..Default::default()
    }
}
