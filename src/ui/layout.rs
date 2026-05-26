// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::components::connection::ConnectionStatus;
use crate::ui::messages::{ConnectionMessage, Message};
use crate::ui::state::AppState;
use crate::ui::theme;
use crate::ui::tokens::{
    LAYOUT_DEVICES_MAX_WIDTH, SPACE_0, SPACE_16, SPACE_24, SPACE_4, SPACE_8, TYPE_BODY,
    TYPE_CAPTION, TYPE_TITLE, WINDOW_MEDIUM_MAX, WINDOW_NARROW_MAX,
};
use crate::ui::views;
use iced::widget::text;
use iced::{
    widget::{column, container, responsive, row, scrollable},
    Element, Length, Padding,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutBucket {
    Narrow,
    Medium,
    Wide,
}

pub fn layout_bucket_for_width(width: f32) -> LayoutBucket {
    if width <= WINDOW_NARROW_MAX {
        LayoutBucket::Narrow
    } else if width <= WINDOW_MEDIUM_MAX {
        LayoutBucket::Medium
    } else {
        LayoutBucket::Wide
    }
}

pub fn views_for_bucket(bucket: LayoutBucket) -> Vec<&'static str> {
    match bucket {
        LayoutBucket::Narrow => vec![
            "header",
            "status",
            "graph",
            "presets",
            "autoeq",
            "advanced",
            "diagnostics",
        ],
        LayoutBucket::Medium => vec![
            "header",
            "status",
            "graph",
            "autoeq+presets",
            "advanced",
            "diagnostics",
        ],
        LayoutBucket::Wide => vec!["header+status", "left:graph+advanced", "right:tools"],
    }
}

pub fn view_narrow(state: &AppState) -> Element<'_, Message> {
    scrollable(
        column![
            views::graph_panel::view_graph(state),
            views::preamp::view_preamp(state),
            views::bands::view_bands(state),
            views::tools_panel::view_tools_panel(state),
            views::diagnostics::view_diagnostics_section(state),
        ]
        .spacing(SPACE_16)
        .width(Length::Fill),
    )
    .into()
}

pub fn view_medium(state: &AppState) -> Element<'_, Message> {
    use crate::ui::tokens::{GRAPH_HEIGHT_MEDIUM, WINDOW_MAX_CONTENT_WIDTH};

    let graph_section = container(views::graph_panel::view_graph(state))
        .height(Length::Fixed(GRAPH_HEIGHT_MEDIUM))
        .width(Length::Fill);

    let left_column = column![
        graph_section,
        views::preamp::view_preamp(state),
        scrollable(views::bands::view_bands(state))
            .height(Length::Fill)
            .width(Length::Fill),
    ]
    .spacing(SPACE_8)
    .width(Length::FillPortion(3));

    let right_column = scrollable(
        column![
            views::tools_panel::view_tools_panel(state),
            views::diagnostics::view_diagnostics_section(state),
        ]
        .spacing(SPACE_16)
        .padding(Padding {
            top: SPACE_0,
            right: SPACE_8,
            bottom: SPACE_0,
            left: SPACE_0,
        }),
    )
    .width(Length::FillPortion(2));

    container(
        row![left_column, right_column]
            .spacing(SPACE_16)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .max_width(WINDOW_MAX_CONTENT_WIDTH)
    .center_x(Length::Fill)
    .into()
}

pub fn view_wide(state: &AppState) -> Element<'_, Message> {
    let left_content = column![
        views::graph_panel::view_graph_fill(state),
        views::preamp::view_preamp(state),
        views::bands::view_bands(state),
    ]
    .spacing(SPACE_8)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding {
        top: SPACE_16,
        right: SPACE_16,
        bottom: SPACE_8,
        left: SPACE_16,
    });

    let right_sidebar = container(
        scrollable(
            column![
                views::tools_panel::view_tools_panel(state),
                views::diagnostics::view_diagnostics_section(state),
            ]
            .spacing(SPACE_16)
            .padding(Padding {
                top: SPACE_16,
                right: SPACE_16,
                bottom: SPACE_16,
                left: SPACE_0,
            }),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(crate::ui::tokens::SIDEBAR_WIDTH));

    row![left_content, right_sidebar]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn view_disconnected(state: &AppState) -> Element<'_, Message> {
    let mut devices_col = column![text("Available Devices")
        .size(TYPE_TITLE)
        .color(crate::ui::tokens::COLOR_ON_SURFACE),]
    .spacing(SPACE_16);

    if state.connection.available_devices.is_empty() {
        devices_col = devices_col.push(
            text("No devices found. Is your DAC plugged in?")
                .size(TYPE_BODY)
                .color(crate::ui::tokens::COLOR_ON_SURFACE_VARIANT),
        );
    } else {
        for dev in state.connection.available_devices.iter() {
            let name = crate::hardware::get_profile(dev.vendor_id, dev.product_id)
                .map(|p| p.name())
                .unwrap_or("Unknown Device");

            let dev_row = row![column![
                text(name)
                    .size(TYPE_BODY)
                    .color(crate::ui::tokens::COLOR_ON_SURFACE),
                text(format!(
                    "VID: {:04X}  PID: {:04X}",
                    dev.vendor_id, dev.product_id
                ))
                .size(TYPE_CAPTION)
                .color(crate::ui::tokens::COLOR_ON_SURFACE_VARIANT)
            ]
            .spacing(SPACE_4)];

            let dev_btn =
                iced::widget::button(container(dev_row).padding(SPACE_16).width(Length::Fill))
                    .style(theme::device_button_style)
                    .on_press(Message::Connection(ConnectionMessage::ConnectPressed(
                        dev.clone(),
                    )))
                    .width(Length::Fill);

            devices_col = devices_col.push(dev_btn);
        }
    }

    container(devices_col.max_width(LAYOUT_DEVICES_MAX_WIDTH))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(SPACE_24)
        .into()
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Element<'_, Message> = if state.connection.status == ConnectionStatus::Disconnected
    {
        container(view_disconnected(state))
            .padding(SPACE_24)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        responsive(move |size| {
            let bucket = layout_bucket_for_width(size.width);
            match bucket {
                LayoutBucket::Narrow => container(view_narrow(state))
                    .padding(SPACE_16)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                LayoutBucket::Medium => container(view_medium(state))
                    .padding(SPACE_16)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                LayoutBucket::Wide => view_wide(state),
            }
        })
        .into()
    };

    let main_view = column![
        views::header::view_header(state),
        views::status_banner::view_status_banner(state),
        content,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    crate::ui::modals::with_modal_overlay(state, main_view)
}
