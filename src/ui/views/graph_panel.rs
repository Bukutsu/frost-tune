use crate::ui::graph::EqGraph;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::SPACE_12;
use iced::widget::{canvas, container, responsive};
use iced::{Element, Length};

pub fn view_graph(state: &MainWindow) -> Element<'_, Message> {
    responsive(move |size| {
        let height = if size.width < 1000.0 {
            260.0
        } else if size.width < 1280.0 {
            300.0
        } else {
            340.0
        };

        container(
            canvas(EqGraph::new(
                &state.editor_state.filters,
                state.editor_state.global_gain,
            ))
            .width(Length::Fill)
            .height(Length::Fixed(height)),
        )
        .padding(SPACE_12)
        .style(|_theme| container::Style {
            background: Some(theme::GRAPH_BG.into()),
            border: iced::Border {
                color: iced::Color { a: 0.2, ..theme::TOKYO_NIGHT_TERMINAL_BLACK },
                width: 1.0,
                radius: theme::CARD_RADIUS.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
    })
    .into()
}
