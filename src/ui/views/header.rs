// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::models::Device;
use crate::ui::messages::Message;
use crate::ui::state::{ConnectionStatus, MainWindow};
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_INFO, COLOR_ON_SURFACE_VARIANT, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_WARNING,
    LAYOUT_HEADER_BUSY_WIDTH, SPACE_12, SPACE_16, SPACE_2, SPACE_8, TYPE_BODY, TYPE_CAPTION,
    TYPE_TINY, TYPE_TITLE,
};
use crate::ui::views::toolbar_button;
use iced::font::Weight;
use iced::widget::{column, container, row, text, tooltip};
use iced::{Element, Font, Length};

fn sync_toolbar_button<'a>(
    label: &'a str,
    on_press: Message,
    action: &'a str,
    state: &'a MainWindow,
    is_busy: bool,
    is_connected: bool,
    shortcut: &'a str,
) -> Element<'a, Message> {
    if !is_busy && is_connected {
        let btn = toolbar_button(label)
            .on_press(on_press)
            .style(theme::m3_tonal_button);
        Element::from(
            tooltip(
                btn,
                text(format!("{} ({})", label, shortcut)).size(TYPE_TINY),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
    } else {
        let btn = toolbar_button(label).style(theme::m3_tonal_button);
        if let Some(reason) = state.disabled_reason_for_action(action) {
            Element::from(
                tooltip(btn, text(reason), tooltip::Position::Bottom).style(theme::tooltip_style),
            )
        } else {
            Element::from(btn)
        }
    }
}

pub fn view_header(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling
        || state.operation_lock.is_pushing
        || state.operation_lock.is_connecting
        || state.operation_lock.is_disconnecting;

    let is_connected = state.connection_status == ConnectionStatus::Connected;

    let mut sync_buttons = row![
        sync_toolbar_button(
            "Read",
            Message::PullPressed,
            "read",
            state,
            is_busy,
            is_connected,
            "Ctrl+R"
        ),
        if !is_busy && is_connected && !state.editor_state.session.input_buffer.has_errors() {
            let btn = toolbar_button("Write")
                .on_press(Message::PushPressed)
                .style(theme::m3_filled_button);
            Element::from(
                tooltip(
                    btn,
                    text("Write to device (Enter)").size(TYPE_TINY),
                    tooltip::Position::Bottom,
                )
                .style(theme::tooltip_style),
            )
        } else {
            let btn = toolbar_button("Write").style(theme::m3_filled_button);
            if let Some(reason) = state.disabled_reason_for_action("write") {
                Element::from(
                    tooltip(btn, text(reason), tooltip::Position::Bottom)
                        .style(theme::tooltip_style),
                )
            } else {
                Element::from(btn)
            }
        },
    ]
    .spacing(SPACE_8);

    if state.connection_status != ConnectionStatus::Disconnected {
        sync_buttons = sync_buttons.push(sync_toolbar_button(
            "Disconnect",
            Message::DisconnectPressed,
            "disconnect",
            state,
            is_busy,
            is_connected,
            "",
        ));
    }

    let status_indicator: Element<'_, Message> = match &state.connection_status {
        ConnectionStatus::Connected => text("• SYNCED").size(TYPE_TINY).color(COLOR_SUCCESS).into(),
        ConnectionStatus::Connecting => text("• CONNECTING")
            .size(TYPE_TINY)
            .color(COLOR_WARNING)
            .into(),
        _ => text("• OFFLINE")
            .size(TYPE_TINY)
            .color(COLOR_ON_SURFACE_VARIANT)
            .into(),
    };

    let device_info: Element<'_, Message> = if let Some(ref dev) = state.connected_device {
        let device_type = Device::from_vid_pid(dev.vendor_id, dev.product_id);
        tooltip(
            text(device_type.name())
                .size(TYPE_BODY)
                .color(COLOR_PRIMARY)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
            text(format!(
                "VID:{:04X}  PID:{:04X}  Path: {}",
                dev.vendor_id, dev.product_id, dev.path
            ))
            .size(TYPE_CAPTION),
            tooltip::Position::Bottom,
        )
        .style(theme::tooltip_style)
        .into()
    } else {
        row![].into()
    };

    container(
        row![
            column![
                row![
                    text("Frost-Tune")
                        .size(TYPE_TITLE)
                        .color(COLOR_PRIMARY)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        }),
                    status_indicator,
                ]
                .spacing(SPACE_12)
                .align_y(iced::Alignment::Center),
                device_info,
            ]
            .spacing(SPACE_2),
            container(text("")).width(Length::Fill),
            container(if is_busy {
                Element::from(text("Device busy…").size(TYPE_CAPTION).color(COLOR_INFO))
            } else {
                Element::from(text(""))
            })
            .width(Length::Fixed(LAYOUT_HEADER_BUSY_WIDTH))
            .align_x(iced::Alignment::End),
            sync_buttons,
        ]
        .padding(SPACE_16)
        .spacing(SPACE_16)
        .align_y(iced::Alignment::Center),
    )
    .style(theme::header_card_style)
    .width(Length::Fill)
    .into()
}
