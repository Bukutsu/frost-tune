// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::components::connection::ConnectionStatus;
use crate::ui::components::editor::EqSource;
use crate::ui::messages::*;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{
    COLOR_ERROR, COLOR_INFO, COLOR_ON_SURFACE_VARIANT, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_WARNING,
    SPACE_12, SPACE_16, SPACE_2, SPACE_8, TYPE_BODY, TYPE_CAPTION, TYPE_TINY, TYPE_TITLE,
};
use crate::ui::views::toolbar_button;
use iced::font::Weight;
use iced::widget::{button, column, container, row, text, tooltip};
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
    let is_busy = state.connection.operation_lock.is_pulling
        || state.connection.operation_lock.is_pushing
        || state.connection.operation_lock.is_connecting
        || state.connection.operation_lock.is_disconnecting;

    let is_connected = state.connection.status == ConnectionStatus::Connected;

    let mut sync_buttons = row![
        sync_toolbar_button(
            "Pull",
            Message::Editor(EditorMessage::PullPressed),
            "read",
            state,
            is_busy,
            is_connected,
            "Ctrl+R"
        ),
        if !is_busy && is_connected && !state.editor.session.input_buffer.has_errors() {
            let btn = toolbar_button("Push")
                .on_press(Message::Editor(EditorMessage::PushPressed))
                .style(theme::m3_filled_button);
            Element::from(
                tooltip(
                    btn,
                    text("Push to device (Enter)").size(TYPE_TINY),
                    tooltip::Position::Bottom,
                )
                .style(theme::tooltip_style),
            )
        } else {
            let btn = toolbar_button("Push").style(theme::m3_filled_button);
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

    if state.connection.status != ConnectionStatus::Disconnected {
        sync_buttons = sync_buttons.push(sync_toolbar_button(
            "Disconnect",
            Message::Connection(ConnectionMessage::DisconnectPressed),
            "disconnect",
            state,
            is_busy,
            is_connected,
            "",
        ));
    }

    let (chip_label, chip_color) = if state.connection.operation_lock.is_pulling {
        ("Pulling…", COLOR_INFO)
    } else if state.connection.operation_lock.is_pushing {
        ("Pushing…", COLOR_INFO)
    } else if state.connection.operation_lock.is_disconnecting {
        ("Disconnecting…", COLOR_WARNING)
    } else {
        match &state.connection.status {
            ConnectionStatus::Connected => ("Synced", COLOR_SUCCESS),
            ConnectionStatus::Connecting => ("Connecting…", COLOR_WARNING),
            ConnectionStatus::Disconnected => ("Offline", COLOR_ON_SURFACE_VARIANT),
            ConnectionStatus::Error(_) => ("Error", COLOR_ERROR),
        }
    };

    let status_indicator: Element<'_, Message> = text(format!("● {}", chip_label))
        .size(TYPE_TINY)
        .color(chip_color)
        .into();

    let error_count = state.diagnostics.errors().count();
    let diag_pill: Option<Element<'_, Message>> =
        if error_count > 0 && !state.editor.ui.show_diagnostics {
            let pill_btn = button(
                text(format!(
                    "⚠ {} error{}",
                    error_count,
                    if error_count == 1 { "" } else { "s" }
                ))
                .size(TYPE_TINY)
                .color(COLOR_ERROR),
            )
            .padding([SPACE_2, SPACE_8])
            .on_press(Message::Diagnostics(DiagnosticsMessage::ToggleDiagnostics))
            .style(theme::m3_text_button);
            Some(
                tooltip(
                    pill_btn,
                    text("Open diagnostics").size(TYPE_TINY),
                    tooltip::Position::Bottom,
                )
                .style(theme::tooltip_style)
                .into(),
            )
        } else {
            None
        };

    let profile_name = match state.editor.ui.eq_source {
        EqSource::Profile => state
            .editor
            .ui
            .selected_profile_name
            .as_deref()
            .unwrap_or("Profile"),
        EqSource::Pulled => "Pulled from device",
        EqSource::Imported => "Imported",
        EqSource::Default => "Default EQ",
    };

    let mut display_row = row![text(format!("— {}", profile_name))
        .size(TYPE_BODY)
        .color(COLOR_ON_SURFACE_VARIANT)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center);

    if state.editor.session.is_dirty {
        display_row = display_row.push(
            container(
                text("UNSAVED")
                    .size(TYPE_TINY)
                    .color(COLOR_WARNING)
                    .font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    }),
            )
            .padding([SPACE_2, SPACE_8])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.1,
                    ..COLOR_WARNING
                })),
                border: iced::Border {
                    color: COLOR_WARNING,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
        );
    }
    let profile_display: Element<'_, Message> = display_row.into();

    let device_info: Element<'_, Message> = if let Some(ref dev) = state.connection.connected_device
    {
        let name = crate::hardware::get_profile(dev.vendor_id, dev.product_id)
            .map(|p| p.name())
            .unwrap_or("Unknown Device");
        tooltip(
            text(name).size(TYPE_BODY).color(COLOR_PRIMARY).font(Font {
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

    let mut status_row = row![
        text("Frost-Tune")
            .size(TYPE_TITLE)
            .color(COLOR_PRIMARY)
            .font(Font {
                weight: Weight::Bold,
                ..Default::default()
            }),
        profile_display,
        status_indicator,
    ]
    .spacing(SPACE_12)
    .align_y(iced::Alignment::Center);

    if let Some(pill) = diag_pill {
        status_row = status_row.push(pill);
    }

    container(
        row![
            column![status_row, device_info,].spacing(SPACE_2),
            container(text("")).width(Length::Fill),
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
