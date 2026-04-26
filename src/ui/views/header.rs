use crate::models::Device;
use crate::ui::messages::Message;
use crate::ui::state::{ConnectionStatus, DisconnectReason, MainWindow};
use crate::ui::theme::{self, TOKYO_NIGHT_ERROR, TOKYO_NIGHT_FG, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_SUCCESS, TOKYO_NIGHT_WARNING, TOKYO_NIGHT_BLUE, TOKYO_NIGHT_YELLOW};
use crate::ui::tokens::{SPACE_16, SPACE_4, SPACE_8, SPACE_12, TYPE_BODY, TYPE_CAPTION, TYPE_DISPLAY, TYPE_LABEL};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn view_header(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling
        || state.operation_lock.is_pushing
        || state.operation_lock.is_connecting
        || state.operation_lock.is_disconnecting;

    let status_text = match &state.connection_status {
        ConnectionStatus::Disconnected => match state.disconnect_reason {
            DisconnectReason::None => text("Disconnected").color(TOKYO_NIGHT_MUTED),
            DisconnectReason::Manual => text("Disconnected (by user)").color(TOKYO_NIGHT_MUTED),
            DisconnectReason::DeviceLost => {
                text("Disconnected (device unplugged)").color(TOKYO_NIGHT_WARNING)
            }
            DisconnectReason::Error(ref e) => {
                text(format!("Error: {}", e)).color(TOKYO_NIGHT_ERROR)
            }
        },
        ConnectionStatus::Connecting => text("Connecting...").color(TOKYO_NIGHT_YELLOW),
        ConnectionStatus::Connected => text("Connected").color(TOKYO_NIGHT_SUCCESS),
        ConnectionStatus::Error(e) => text(format!("Error: {}", e)).color(TOKYO_NIGHT_ERROR),
    };

    let device_info_text = if let Some(ref dev) = state.connected_device {
        let device_type = Device::from_vid_pid(dev.vendor_id, dev.product_id);
        let name = device_type.name();
        let vid_pid = format!("VID:{:04X} PID:{:04X}", dev.vendor_id, dev.product_id);
        let mfr = dev
            .manufacturer
            .as_deref()
            .map(|m| format!(" ({})", m))
            .unwrap_or_default();
        let row_el: Element<'_, Message> = column![
            text(format!("Device: {}", name))
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_PRIMARY),
            text(format!("{}{}", vid_pid, mfr))
                .size(TYPE_CAPTION)
                .color(TOKYO_NIGHT_MUTED),
        ]
        .spacing(SPACE_4)
        .into();
        row_el
    } else {
        text("No device connected")
            .size(TYPE_LABEL)
            .color(TOKYO_NIGHT_MUTED)
            .into()
    };

    let btn_row = row![
        if !is_busy
            && (state.connection_status == ConnectionStatus::Disconnected
                || matches!(&state.connection_status, ConnectionStatus::Error(_)))
        {
            action_button("Connect")
                .on_press(Message::ConnectPressed)
                .style(theme::pill_primary_button)
        } else {
            action_button("Connect").style(theme::pill_primary_button)
        },
        if !is_busy && state.connection_status == ConnectionStatus::Connected {
            action_button("Disconnect")
                .on_press(Message::DisconnectPressed)
                .style(theme::pill_secondary_button)
        } else {
            action_button("Disconnect").style(theme::pill_secondary_button)
        },
        if !is_busy && state.connection_status == ConnectionStatus::Connected {
            action_button("Read Device")
                .on_press(Message::PullPressed)
                .style(theme::pill_secondary_button)
        } else {
            action_button("Read Device").style(theme::pill_secondary_button)
        },
        if !is_busy && state.connection_status == ConnectionStatus::Connected {
            action_button("Write Device")
                .on_press(Message::PushPressed)
                .style(theme::pill_primary_button)
        } else {
            action_button("Write Device").style(theme::pill_primary_button)
        },
    ]
    .spacing(SPACE_8);

    let loading_indicator = if is_busy {
        row![
            text("Processing...")
                .size(TYPE_CAPTION)
                .color(TOKYO_NIGHT_BLUE),
        ]
        .spacing(SPACE_8)
        .align_y(iced::Alignment::Center)
    } else {
        row![]
    };

    container(
        row![
            column![
                text("Frost-Tune")
                    .size(TYPE_DISPLAY)
                    .color(TOKYO_NIGHT_PRIMARY),
                device_info_text,
                text("Workflow: Connect → Read Device → Edit → Write Device")
                    .size(TYPE_LABEL)
                    .color(TOKYO_NIGHT_FG),
                row![status_text.size(TYPE_BODY), loading_indicator].spacing(SPACE_16),
            ]
            .spacing(SPACE_4),
            container(text("")).width(Length::Fill),
            btn_row,
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding(SPACE_12)
    .style(theme::header_card_style)
    .into()
}
