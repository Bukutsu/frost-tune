use crate::ui::graph::EqGraph;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_16};
use iced::widget::{canvas, container, responsive};
use iced::{Element, Length, Padding};

/// Graph with a fixed height based on width — for scrollable layouts (narrow, medium).
pub fn view_graph(state: &MainWindow) -> Element<'_, Message> {
    responsive(move |size| {
        let height = if size.width < 600.0 {
            240.0
        } else if size.width < 1000.0 {
            280.0
        } else {
            320.0
        };

        container(
            canvas(EqGraph::new(
                &state.editor_state.filters,
                state.editor_state.global_gain,
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
            &state.editor_state.filters,
            state.editor_state.global_gain,
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
        background: Some(theme::GRAPH_BG.into()),
        border: iced::Border {
            color: iced::Color {
                a: 0.2,
                ..theme::TOKYO_NIGHT_TERMINAL_BLACK
            },
            width: 1.0,
            radius: theme::CARD_RADIUS.into(),
        },
        ..Default::default()
    }
}
