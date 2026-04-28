use crate::diagnostics::{DiagnosticEvent, DiagnosticsStore, LogLevel, Source};
use crate::hardware::worker::{UsbWorker};
use crate::models::Filter;
use crate::ui::messages::{Message, StatusMessage, StatusSeverity};
use crate::ui::state::{
    ConfirmAction, ConnectionStatus, DisconnectReason, EditorState, InputBuffer, MainWindow,
    OperationLock,
};
use crate::ui::theme;
use crate::ui::tokens::{SPACE_8, SPACE_16, SPACE_24, SPACE_4, TYPE_BODY, TYPE_CAPTION, TYPE_TITLE, WINDOW_MEDIUM_MAX, WINDOW_NARROW_MAX};
use crate::ui::views;
use iced::widget::text;
use iced::{
    widget::{column, container, responsive, row, scrollable},
    Element, Length, Padding, Subscription, Task,
};
use std::sync::Arc;

pub const STATUS_AUTO_CLEAR_SECS: u64 = 5;
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

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

pub fn parse_freq_string(s: &str) -> Option<u16> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let mut multiplier = 1.0;
    let mut num_str = s.as_str();

    if s.ends_with('k') {
        multiplier = 1000.0;
        num_str = &s[..s.len() - 1].trim();
    } else if s.ends_with("hz") {
        num_str = &s[..s.len() - 2].trim();
    }

    if let Ok(v) = num_str.parse::<f64>() {
        let hz = (v * multiplier).round() as u16;
        if hz >= 20 && hz <= 20000 {
            return Some(hz);
        }
    }
    None
}

impl MainWindow {
    fn new() -> (Self, Task<Message>) {
        let worker = Arc::new(UsbWorker::new());
        let default_filters: Vec<Filter> =
            (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let window = MainWindow {
            connection_status: ConnectionStatus::Disconnected,
            disconnect_reason: DisconnectReason::None,
            editor_state: EditorState {
                filters: default_filters.clone(),
                global_gain: 0,
                status_message: None,
                diagnostics_errors_only: false,
                profiles: Vec::new(),
                selected_profile_name: None,
                new_profile_name: String::new(),
                input_buffer: InputBuffer::default(),

                pending_confirm: ConfirmAction::None,
            },
            operation_lock: OperationLock::default(),
            worker: Some(worker),
            connected_device: None,
            available_devices: Vec::new(),
            selected_device_index: None,
            diagnostics: DiagnosticsStore::default(),
        };
        let load_profiles_task = Task::perform(
            async move { crate::storage::load_all_profiles().unwrap_or_default() },
            Message::ProfilesLoaded,
        );
        (
            window,
            load_profiles_task,
        )
    }

    fn title(&self) -> String {
        "Frost-Tune".into()
    }

    fn app_theme(_state: &Self) -> iced::Theme {
        theme::theme()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        crate::ui::update::update(self, message)
    }
    pub fn set_status(
        &mut self,
        content: impl Into<String>,
        severity: StatusSeverity,
    ) -> Task<Message> {
        let content = content.into();
        let skip_diag_echo = content.starts_with("Loaded profile:")
            || content.starts_with("Saved profile:")
            || content.starts_with("Deleted profile:")
            || content.starts_with("Imported ")
            || content.starts_with("Exported ");

        if !skip_diag_echo {
            self.diagnostics.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Status set: {}", content),
            ));
        }
        let should_auto_clear = self.status_should_auto_clear(severity);
        self.editor_state.status_message = Some(StatusMessage {
            content,
            severity,
            created_at: chrono::Local::now().to_rfc3339(),
        });
        if should_auto_clear {
            Task::perform(
                async { tokio::time::sleep(Self::status_auto_clear_duration()).await },
                |_| Message::ClearStatusMessage,
            )
        } else {
            Task::none()
        }
    }

    pub fn status_auto_clear_duration() -> std::time::Duration {
        std::time::Duration::from_secs(STATUS_AUTO_CLEAR_SECS)
    }

    pub fn status_should_auto_clear(&self, severity: StatusSeverity) -> bool {
        if self.operation_lock.is_connecting
            || self.operation_lock.is_disconnecting
            || self.operation_lock.is_pulling
            || self.operation_lock.is_pushing
        {
            return false;
        }
        matches!(severity, StatusSeverity::Info | StatusSeverity::Success)
    }

    pub fn header_status_message(&self) -> String {
        match &self.connection_status {
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
            ConnectionStatus::Connecting => "Connecting...".to_string(),
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Error(e) => format!("Error: {}", e),
        }
    }

    pub fn status_banner_message(&self) -> Option<String> {
        self.editor_state
            .status_message
            .as_ref()
            .map(|m| m.content.clone())
    }

    pub fn disabled_reason_for_action(&self, action: &str) -> Option<String> {
        if let ConnectionStatus::Error(e) = &self.connection_status {
            return Some(format!("Error: {}", e));
        }

        match action {
            "connect" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    None
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else {
                    Some("Device already connected or in error".to_string())
                }
            }
            "disconnect" => {
                if self.operation_lock.is_disconnecting {
                    Some("Disconnecting...".to_string())
                } else if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else {
                    None
                }
            }
            "read" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else if self.operation_lock.is_pulling {
                    Some("Operation in progress: Reading".to_string())
                } else if self.operation_lock.is_pushing {
                    Some("Operation in progress: Writing or Connecting".to_string())
                } else {
                    None
                }
            }
            "write" => {
                if self.connection_status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.operation_lock.is_connecting
                    || self.connection_status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device...".to_string())
                } else if self.operation_lock.is_pushing {
                    Some("Operation in progress: Writing".to_string())
                } else if self.operation_lock.is_pulling {
                    Some("Operation in progress: Reading or Connecting".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn header_disabled_reason_message(&self) -> Option<String> {
        if self.operation_lock.is_disconnecting {
            return Some("Disconnecting...".to_string());
        }
        if self.operation_lock.is_connecting {
            return Some("Connecting to device...".to_string());
        }
        if self.operation_lock.is_pulling {
            return Some("Operation in progress: Reading".to_string());
        }
        if self.operation_lock.is_pushing {
            return Some("Operation in progress: Writing or Connecting".to_string());
        }
        if let ConnectionStatus::Error(e) = &self.connection_status {
            return Some(format!("Error: {}", e));
        }
        if self.connection_status == ConnectionStatus::Disconnected {
            return Some("Device disconnected".to_string());
        }
        None
    }

    pub fn views_for_bucket(&self, bucket: LayoutBucket) -> Vec<&'static str> {
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

    fn view_narrow(&self) -> Element<'_, Message> {
        scrollable(
            column![
                views::graph_panel::view_graph(self),
                views::presets_preamp::view_presets_and_preamp(
                    self,
                    views::presets_preamp::PresetsLayout::Narrow,
                ),
                views::autoeq::view_autoeq(self),
                views::bands::view_bands(self),
                views::diagnostics::view_diagnostics_section(self),
            ]
            .spacing(SPACE_16)
            .width(Length::Fill)
        )
        .into()
    }

    fn view_medium(&self) -> Element<'_, Message> {
        let tools_row = row![
            container(views::presets_preamp::view_presets_and_preamp(
                self,
                views::presets_preamp::PresetsLayout::Medium,
            ))
            .width(Length::FillPortion(1))
            .height(Length::Fill),
            container(views::autoeq::view_autoeq(self))
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        ]
        .spacing(SPACE_16)
        .align_y(iced::Alignment::Start)
        .width(Length::Fill);

        scrollable(
            column![
                views::graph_panel::view_graph(self),
                tools_row,
                views::bands::view_bands(self),
                views::diagnostics::view_diagnostics_section(self),
            ]
            .spacing(SPACE_16)
            .width(Length::Fill)
        )
        .into()
    }

    fn view_wide(&self) -> Element<'_, Message> {
        let left_content = column![
            views::graph_panel::view_graph_fill(self),
            views::bands::view_bands(self),
        ]
        .spacing(SPACE_8)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: 0.0, right: SPACE_16, bottom: SPACE_8, left: SPACE_16 });

        let right_sidebar = container(
            scrollable(
                column![
                    views::presets_preamp::view_presets_and_preamp(
                        self,
                        views::presets_preamp::PresetsLayout::Narrow,
                    ),
                    views::autoeq::view_autoeq(self),
                    views::diagnostics::view_diagnostics_section(self),
                ]
                .spacing(SPACE_16)
                .padding(Padding { top: 0.0, right: SPACE_16, bottom: SPACE_16, left: 0.0 })
            )
            .height(Length::Fill)
        )
        .width(Length::Fixed(crate::ui::tokens::SIDEBAR_WIDTH));

        row![left_content, right_sidebar]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn with_modal_overlay<'a>(&self, main_view: Element<'a, Message>) -> Element<'a, Message> {
        if let Some(dialog) = match self.editor_state.pending_confirm {
            ConfirmAction::ResetFilters => Some(views::confirm_dialog::view_confirm_dialog(
                "Reset Filters?".to_string(),
                "This will reset all 10 bands to default values and set global gain to 0.".to_string(),
                "Reset",
                Message::ConfirmResetFilters,
            )),
            ConfirmAction::DeleteProfile => Some(views::confirm_dialog::view_confirm_dialog(
                "Delete Profile?".to_string(),
                "Are you sure you want to delete this profile? This cannot be undone.".to_string(),
                "Delete",
                Message::ConfirmDeleteProfile,
            )),
            ConfirmAction::ElevatedConnect(ref device) => Some(views::confirm_dialog::view_confirm_dialog(
                "Temporary Root Access Required".to_string(),
                format!("Connecting to {}.\n\nOn Linux, temporary root access is required to communicate with USB devices. You will be prompted for your password.", device.manufacturer.as_deref().unwrap_or("Unknown Device")),
                "Continue",
                Message::ConfirmElevatedConnect(device.clone()),
            )),
            ConfirmAction::None => None,
        } {
            iced::widget::stack![
                main_view,
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Color { a: 0.8, ..crate::ui::theme::TOKYO_NIGHT_BG_DARK }.into()),
                        ..Default::default()
                    })
            ]
            .into()
        } else {
            main_view
        }
    }

    fn view_disconnected(&self) -> Element<'_, Message> {
        let mut devices_col = column![
            text("Available Devices")
                .size(TYPE_TITLE)
                .color(crate::ui::theme::TOKYO_NIGHT_FG),
        ].spacing(SPACE_16);

        if self.available_devices.is_empty() {
            devices_col = devices_col.push(
                text("No devices found. Is your DAC plugged in?")
                    .size(TYPE_BODY)
                    .color(crate::ui::theme::TOKYO_NIGHT_MUTED)
            );
        } else {
            for (i, dev) in self.available_devices.iter().enumerate() {
                let is_selected = self.selected_device_index == Some(i);
                
                let bg_color = if is_selected {
                    crate::ui::theme::TOKYO_NIGHT_BG_HIGHLIGHT
                } else {
                    crate::ui::theme::TOKYO_NIGHT_BG_DARK
                };
                
                let border_color = if is_selected {
                    crate::ui::theme::TOKYO_NIGHT_PRIMARY
                } else {
                    iced::Color::TRANSPARENT
                };

                let dev_type = crate::models::Device::from_vid_pid(dev.vendor_id, dev.product_id);
                
                let dev_row = row![
                    column![
                        text(dev_type.name())
                            .size(TYPE_BODY)
                            .color(crate::ui::theme::TOKYO_NIGHT_FG),
                        text(format!("VID: {:04X}  PID: {:04X}  Path: {}", dev.vendor_id, dev.product_id, dev.path))
                            .size(TYPE_CAPTION)
                            .color(crate::ui::theme::TOKYO_NIGHT_MUTED)
                    ].spacing(SPACE_4)
                ];

                let dev_btn = iced::widget::button(
                    container(dev_row)
                        .padding(SPACE_16)
                        .width(Length::Fill)
                )
                .style(move |_theme, _status| iced::widget::button::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: border_color,
                    },
                    text_color: crate::ui::theme::TOKYO_NIGHT_FG,
                    ..Default::default()
                })
                .on_press(Message::DeviceSelected(i))
                .width(Length::Fill);

                devices_col = devices_col.push(dev_btn);
            }
        }
        
        let mut connect_col = column![devices_col].spacing(SPACE_24);
        
        if let Some(idx) = self.selected_device_index {
            if let Some(dev) = self.available_devices.get(idx) {
                let connect_btn = crate::ui::views::action_button("Connect")
                    .style(crate::ui::theme::pill_primary_button)
                    .on_press(Message::ConnectPressed(dev.clone()));
                connect_col = connect_col.push(connect_btn);
            }
        }

        container(
            scrollable(connect_col)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = if self.connection_status == ConnectionStatus::Disconnected {
            container(self.view_disconnected())
                .padding(SPACE_24)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            responsive(move |size| {
                let bucket = layout_bucket_for_width(size.width);
                match bucket {
                    LayoutBucket::Narrow => container(self.view_narrow())
                        .padding(SPACE_16)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into(),
                    LayoutBucket::Medium => container(self.view_medium())
                        .padding(SPACE_24)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into(),
                    LayoutBucket::Wide => self.view_wide(),
                }
            }).into()
        };

        let main_view = column![
            views::header::view_header(self),
            views::status_banner::view_status_banner(self),
            content,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        self.with_modal_overlay(main_view)
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::pin::Pin;
        use std::time::Duration;
        async fn tick() -> Message {
            Message::Tick(std::time::Instant::now())
        }
        time::repeat(|| Pin::from(Box::pin(tick())), Duration::from_secs(2))
    }
}

pub fn run() -> iced::Result {
    iced::application(MainWindow::new, MainWindow::update, MainWindow::view)
        .title(MainWindow::title)
        .subscription(MainWindow::subscription)
        .theme(MainWindow::app_theme)
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_freq_string() {
        assert_eq!(parse_freq_string("1k"), Some(1000));
        assert_eq!(parse_freq_string("100hz"), Some(100));
        assert_eq!(parse_freq_string("20"), Some(20));
        // Testing out of range values
        assert_eq!(parse_freq_string("21k"), None);
        assert_eq!(parse_freq_string("10"), None);
        // Invalid strings
        assert_eq!(parse_freq_string(""), None);
        assert_eq!(parse_freq_string("abc"), None);
        assert_eq!(parse_freq_string("k"), None);
    }
}
