use crate::models::Device;
use crate::ui::messages::Message;
use crate::ui::state::{ConnectionStatus, MainWindow};
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY};
use crate::ui::tokens::{SPACE_16, SPACE_4, SPACE_8, SPACE_12, SPACE_2, SPACE_6, TYPE_BODY, TYPE_CAPTION, TYPE_TITLE, TYPE_TINY};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, text};
use iced::{Element, Font, Length};
use iced::font::Weight;

pub fn view_header(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling
        || state.operation_lock.is_pushing
        || state.operation_lock.is_connecting
        || state.operation_lock.is_disconnecting;

    let is_connected = state.connection_status == ConnectionStatus::Connected;

    let sync_buttons = row![
        if !is_busy && is_connected {
            action_button("Read")
                .on_press(Message::PullPressed)
                .style(theme::pill_secondary_button)
        } else {
            action_button("Read").style(theme::pill_secondary_button)
        },
        if !is_busy && is_connected {
            action_button("Write")
                .on_press(Message::PushPressed)
                .style(|theme, status| {
                    let mut s = theme::pill_primary_button(theme, status);
                    if matches!(status, iced::widget::button::Status::Active) {
                        s.background = Some(iced::Background::Color(theme::ACCENT_VIBRANT));
                        s.text_color = theme::TOKYO_NIGHT_BG_DARK;
                    }
                    s
                })
        } else {
            action_button("Write").style(theme::pill_primary_button)
        },
        if !is_busy && is_connected {
             action_button("Disconnect")
                .on_press(Message::DisconnectPressed)
                .style(theme::pill_secondary_button)
        } else {
            action_button("Disconnect").style(theme::pill_secondary_button)
        },
    ]
    .spacing(SPACE_8);

    let status_indicator = match &state.connection_status {
        ConnectionStatus::Connected => container(text("SYNCED").size(TYPE_TINY).color(theme::TOKYO_NIGHT_BG_DARK))
            .padding([SPACE_2, SPACE_6])
            .style(|_theme| container::Style {
                background: Some(theme::TOKYO_NIGHT_SUCCESS.into()),
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..Default::default()
            }),
        ConnectionStatus::Connecting => container(text("CONNECTING").size(TYPE_TINY).color(theme::TOKYO_NIGHT_BG_DARK))
            .padding([SPACE_2, SPACE_6])
            .style(|_theme| container::Style {
                background: Some(theme::TOKYO_NIGHT_YELLOW.into()),
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..Default::default()
            }),
        _ => container(text("OFFLINE").size(TYPE_TINY).color(theme::TOKYO_NIGHT_FG_DARK))
            .padding([SPACE_2, SPACE_6])
            .style(|_theme| container::Style {
                background: Some(theme::TOKYO_NIGHT_TERMINAL_BLACK.into()),
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..Default::default()
            }),
    };

    let device_info = if let Some(ref dev) = state.connected_device {
        let device_type = Device::from_vid_pid(dev.vendor_id, dev.product_id);
        row![
            text(device_type.name())
                .size(TYPE_BODY)
                .color(TOKYO_NIGHT_PRIMARY)
                .font(Font { weight: Weight::Bold, ..Default::default() }),
            text(format!(" (VID:{:04X} PID:{:04X})", dev.vendor_id, dev.product_id))
                .size(TYPE_CAPTION)
                .color(TOKYO_NIGHT_MUTED),
        ].align_y(iced::Alignment::Center).spacing(SPACE_4)
    } else {
        row![text("No device detected").size(TYPE_BODY).color(TOKYO_NIGHT_MUTED)]
    };

    container(
        row![
            column![
                row![
                    text("Frost-Tune")
                        .size(TYPE_TITLE)
                        .color(TOKYO_NIGHT_PRIMARY)
                        .font(Font { weight: Weight::Bold, ..Default::default() }),
                    status_indicator,
                ].spacing(SPACE_12).align_y(iced::Alignment::Center),
                device_info,
            ].spacing(SPACE_2),
            container(text("")).width(Length::Fill),
            if is_busy {
                 text("Device busy...").size(TYPE_CAPTION).color(theme::TOKYO_NIGHT_BLUE)
            } else {
                text("").size(TYPE_CAPTION)
            },
            sync_buttons,
        ]
        .padding(SPACE_16)
        .align_y(iced::Alignment::Center),
    )
    .style(theme::header_card_style)
    .width(Length::Fill)
    .into()
}
