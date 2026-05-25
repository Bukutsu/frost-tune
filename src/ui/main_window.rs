// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::Filter;
use crate::diagnostics::{
    parse_diagnostic_log_line, DiagnosticEvent, DiagnosticsStore, LogLevel, Source,
};
use crate::hardware::worker::UsbWorker;
use crate::ui::components::connection::{ConnectionComponent, ConnectionStatus};
use crate::ui::components::editor::{ConfirmAction, EditorComponent};
use crate::ui::messages::{
    AutoEqMessage, ConnectionMessage, EditorMessage, Message, ProfilesMessage, StatusMessage,
    StatusSeverity,
};
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{
    LAYOUT_DEVICES_MAX_WIDTH, LAYOUT_WINDOW_MIN_HEIGHT, LAYOUT_WINDOW_MIN_WIDTH, SPACE_0, SPACE_16,
    SPACE_24, SPACE_4, SPACE_8, TYPE_BODY, TYPE_CAPTION, TYPE_TITLE, WINDOW_MEDIUM_MAX,
    WINDOW_NARROW_MAX,
};
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
    let mut num_str: &str = &s;

    if let Some(stripped) = s.strip_suffix("khz") {
        multiplier = 1000.0;
        num_str = stripped.trim();
    } else if let Some(stripped) = s.strip_suffix("hz") {
        num_str = stripped.trim();
    } else if let Some(stripped) = s.strip_suffix('k') {
        multiplier = 1000.0;
        num_str = stripped.trim();
    }

    if num_str.is_empty() {
        return None;
    }

    if let Ok(v) = num_str.parse::<f64>() {
        let hz = (v * multiplier).round() as u16;
        if (20..=20000).contains(&hz) {
            return Some(hz);
        }
    }
    None
}

impl MainWindow {
    fn new_with_diagnostics(diagnostics: DiagnosticsStore) -> (Self, Task<Message>) {
        let worker = Arc::new(UsbWorker::new());
        let default_filters: Vec<Filter> =
            (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let settings = crate::storage::load_settings();
        let window = MainWindow {
            editor: EditorComponent {
                data: crate::ui::components::editor::EditorData {
                    filters: default_filters,
                    ..Default::default()
                },
                ui: crate::ui::components::editor::EditorUI {
                    snap_to_iso_enabled: true,
                    auto_pull_on_connect: settings.auto_pull_on_connect,
                    ..Default::default()
                },
                ..Default::default()
            },
            connection: ConnectionComponent {
                worker: Some(worker),
                ..Default::default()
            },
            diagnostics,
            ..Default::default()
        };
        let load_profiles_task = Task::perform(
            async move { crate::storage::load_all_profiles().await },
            |result| Message::Profiles(ProfilesMessage::ProfilesLoaded(result)),
        );
        let load_font_task =
            iced::font::load(crate::ui::tokens::ICON_FONT_BYTES).map(|_| Message::None);

        (
            window,
            Task::batch(vec![load_profiles_task, load_font_task]),
        )
    }

    fn new() -> (Self, Task<Message>) {
        Self::new_with_diagnostics(DiagnosticsStore::default())
    }

    fn title(&self) -> String {
        if self.editor.session.is_dirty {
            "Frost-Tune *".into()
        } else {
            "Frost-Tune".into()
        }
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
        let should_auto_clear = self.status_should_auto_clear(severity.clone());
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.editor.session.status_message = Some(StatusMessage {
            id: id as usize,
            content,
            severity,
            timestamp: std::time::Instant::now(),
        });
        if should_auto_clear {
            Task::perform(
                async { tokio::time::sleep(Self::status_auto_clear_duration()).await },
                move |_| Message::ClearStatusMessage(id as usize),
            )
        } else {
            Task::none()
        }
    }

    pub fn status_auto_clear_duration() -> std::time::Duration {
        std::time::Duration::from_secs(STATUS_AUTO_CLEAR_SECS)
    }

    pub fn status_should_auto_clear(&self, severity: StatusSeverity) -> bool {
        if self.connection.operation_lock.is_connecting
            || self.connection.operation_lock.is_disconnecting
            || self.connection.operation_lock.is_pulling
            || self.connection.operation_lock.is_pushing
        {
            return false;
        }
        matches!(severity, StatusSeverity::Info | StatusSeverity::Success)
    }

    pub fn header_status_message(&self) -> String {
        match &self.connection.status {
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
            ConnectionStatus::Connecting => "Connecting…".to_string(),
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Error(e) => format!("Error: {}", e),
        }
    }

    pub fn status_banner_message(&self) -> Option<String> {
        self.editor
            .session
            .status_message
            .as_ref()
            .map(|m| m.content.clone())
    }

    pub fn disabled_reason_for_action(&self, action: &str) -> Option<String> {
        if let ConnectionStatus::Error(e) = &self.connection.status {
            return Some(format!("Error: {}", e));
        }

        match action {
            "connect" => {
                if self.connection.status == ConnectionStatus::Disconnected {
                    None
                } else if self.connection.operation_lock.is_connecting
                    || self.connection.status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device…".to_string())
                } else {
                    Some("Device already connected or in error".to_string())
                }
            }
            "disconnect" => {
                if self.connection.operation_lock.is_disconnecting {
                    Some("Disconnecting…".to_string())
                } else if self.connection.status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.connection.operation_lock.is_connecting
                    || self.connection.status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device…".to_string())
                } else {
                    None
                }
            }
            "read" => {
                if self.connection.status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.connection.operation_lock.is_connecting
                    || self.connection.status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device…".to_string())
                } else if self.connection.operation_lock.is_pulling {
                    Some("Pulling…".to_string())
                } else if self.connection.operation_lock.is_pushing {
                    Some("Pushing…".to_string())
                } else {
                    None
                }
            }
            "write" => {
                if self.connection.status == ConnectionStatus::Disconnected {
                    Some("Device disconnected".to_string())
                } else if self.connection.operation_lock.is_connecting
                    || self.connection.status == ConnectionStatus::Connecting
                {
                    Some("Connecting to device…".to_string())
                } else if self.editor.session.input_buffer.has_errors() {
                    Some("Resolve input errors first".to_string())
                } else if self.connection.operation_lock.is_pushing {
                    Some("Pushing…".to_string())
                } else if self.connection.operation_lock.is_pulling {
                    Some("Pulling…".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn header_disabled_reason_message(&self) -> Option<String> {
        if self.connection.operation_lock.is_disconnecting {
            return Some("Disconnecting…".to_string());
        }
        if self.connection.operation_lock.is_connecting {
            return Some("Connecting to device…".to_string());
        }
        if self.connection.operation_lock.is_pulling {
            return Some("Operation in progress: Reading".to_string());
        }
        if self.connection.operation_lock.is_pushing {
            return Some("Operation in progress: Writing or Connecting".to_string());
        }
        if let ConnectionStatus::Error(e) = &self.connection.status {
            return Some(format!("Error: {}", e));
        }
        if self.connection.status == ConnectionStatus::Disconnected {
            return Some("Device disconnected".to_string());
        }
        None
    }

    pub fn num_bands(&self) -> usize {
        self.active_device()
            .map(|p| p.capabilities().num_bands)
            .unwrap_or(crate::core::NUM_BANDS)
    }

    pub fn freq_range(&self) -> (u16, u16) {
        self.active_device()
            .map(|p| p.capabilities().freq_range)
            .unwrap_or((crate::core::MIN_FREQ, crate::core::MAX_FREQ))
    }

    pub fn gain_range(&self) -> (f64, f64) {
        self.active_device()
            .map(|p| p.capabilities().band_gain_range)
            .unwrap_or((crate::core::MIN_BAND_GAIN, crate::core::MAX_BAND_GAIN))
    }

    pub fn q_range(&self) -> (f64, f64) {
        self.active_device()
            .map(|p| p.capabilities().q_range)
            .unwrap_or((crate::core::MIN_Q, crate::core::MAX_Q))
    }

    pub fn supports_per_band_enable(&self) -> bool {
        self.active_device()
            .map(|p| p.capabilities().supports_per_band_enable)
            .unwrap_or(true)
    }

    pub fn supported_filter_types(&self) -> crate::core::FilterTypeFlags {
        self.active_device()
            .map(|p| p.capabilities().supported_filter_types)
            .unwrap_or(
                crate::core::FilterTypeFlags::PEAK
                    | crate::core::FilterTypeFlags::LOW_SHELF
                    | crate::core::FilterTypeFlags::HIGH_SHELF
                    | crate::core::FilterTypeFlags::LOW_PASS
                    | crate::core::FilterTypeFlags::HIGH_PASS,
            )
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
                views::preamp::view_preamp(self),
                views::bands::view_bands(self),
                views::tools_panel::view_tools_panel(self),
                views::diagnostics::view_diagnostics_section(self),
            ]
            .spacing(SPACE_16)
            .width(Length::Fill),
        )
        .into()
    }

    fn view_medium(&self) -> Element<'_, Message> {
        use crate::ui::tokens::{GRAPH_HEIGHT_MEDIUM, WINDOW_MAX_CONTENT_WIDTH};

        let graph_section = container(views::graph_panel::view_graph(self))
            .height(Length::Fixed(GRAPH_HEIGHT_MEDIUM))
            .width(Length::Fill);

        let left_column = column![
            graph_section,
            views::preamp::view_preamp(self),
            scrollable(views::bands::view_bands(self))
                .height(Length::Fill)
                .width(Length::Fill),
        ]
        .spacing(SPACE_8)
        .width(Length::FillPortion(3));

        let right_column = scrollable(
            column![
                views::tools_panel::view_tools_panel(self),
                views::diagnostics::view_diagnostics_section(self),
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

    fn view_wide(&self) -> Element<'_, Message> {
        let left_content = column![
            views::graph_panel::view_graph_fill(self),
            views::preamp::view_preamp(self),
            views::bands::view_bands(self),
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
                    views::tools_panel::view_tools_panel(self),
                    views::diagnostics::view_diagnostics_section(self),
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

    fn with_modal_overlay<'a>(&'a self, main_view: Element<'a, Message>) -> Element<'a, Message> {
        if let Some(dialog) = match self.editor.session.pending_confirm {
            ConfirmAction::ResetFilters => Some(views::confirm_dialog::view_confirm_dialog(
                "Reset Filters?".to_string(),
                "This will reset all 10 bands to default values and set global gain to 0.".to_string(),
                "Reset",
                Message::Editor(EditorMessage::ConfirmResetFilters),
                true,
            )),
            ConfirmAction::DeleteProfile => Some(views::confirm_dialog::view_confirm_dialog(
                "Delete Profile?".to_string(),
                "Are you sure you want to delete this profile? This cannot be undone.".to_string(),
                "Delete",
                Message::Profiles(ProfilesMessage::ConfirmDeleteProfile),
                true,
            )),
            ConfirmAction::ImportAutoEQ { ref data, ref default_name } => {
                let count = data.filters.iter().filter(|f| f.enabled).count();
                let message = format!(
                    "This will import {} filters and set global gain to {:.1}dB.\n\nCurrent unsaved settings will be replaced.",
                    count, data.global_gain,
                );
                Some(views::confirm_dialog::view_import_dialog(
                    "Import Profile".to_string(),
                    message,
                    self.editor.session.import_name_input.as_str(),
                    default_name.as_str(),
                    &self.editor.ui.profiles,
                    self.editor.ui.selected_profile_name.as_deref(),
                    self.editor.session.import_temporary,
                    "Import",
                    Message::AutoEq(AutoEqMessage::ConfirmImportWithName),
                ))
            },
            ConfirmAction::OverwriteProfile { ref name, .. } => Some(views::confirm_dialog::view_confirm_dialog(
                "Overwrite Profile?".to_string(),
                format!("Profile '{}' already exists. Overwrite?", name),
                "Overwrite",
                Message::Profiles(ProfilesMessage::ConfirmOverwriteProfile),
                true,
            )),
            ConfirmAction::PullDevice => Some(views::confirm_dialog::view_confirm_dialog(
                "Pull from Device?".to_string(),
                "You have unsaved changes. Pulling from the device will replace your current editor settings with the hardware configuration. Continue?".to_string(),
                "Discard & Pull",
                Message::Editor(EditorMessage::ConfirmPullPressed),
                false,
            )),
            ConfirmAction::PushToDevice => {
                let active = self.editor.data.filters.iter().filter(|f| f.enabled).count();
                let gain = self.editor.data.global_gain;
                Some(views::confirm_dialog::view_confirm_dialog(
                    "Push to Device?".to_string(),
                    format!(
                        "This will push {} active bands (global gain: {} dB) to the hardware.\n\nThis cannot be undone.",
                        active, gain
                    ),
                    "Push",
                    Message::Editor(EditorMessage::ConfirmPushPressed),
                    false,
                ))
            }
            ConfirmAction::LoadProfile { ref name } => Some(views::confirm_dialog::view_confirm_dialog(
                "Unsaved Changes".to_string(),
                format!("You have unsaved changes. Loading '{}' will replace your current editor settings. Continue?", name),
                "Discard & Load",
                Message::Profiles(ProfilesMessage::ConfirmLoadProfile),
                true,
            )),
            ConfirmAction::ExitWithUnsavedChanges(id) => Some(views::confirm_dialog::view_exit_dialog(
                "Unsaved Changes".to_string(),
                "You have unsaved EQ changes. Save before exiting?".to_string(),
                "Save & Exit",
                Message::SaveAndExit(id),
                "Exit Anyway",
                Message::ConfirmExit(id),
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
                        background: Some(iced::Color { a: 0.8, ..crate::ui::tokens::COLOR_SURFACE_DIM }.into()),
                        ..Default::default()
                    })
            ]
            .into()
        } else {
            main_view
        }
    }

    fn view_disconnected(&self) -> Element<'_, Message> {
        let mut devices_col = column![text("Available Devices")
            .size(TYPE_TITLE)
            .color(crate::ui::tokens::COLOR_ON_SURFACE),]
        .spacing(SPACE_16);

        if self.connection.available_devices.is_empty() {
            devices_col = devices_col.push(
                text("No devices found. Is your DAC plugged in?")
                    .size(TYPE_BODY)
                    .color(crate::ui::tokens::COLOR_ON_SURFACE_VARIANT),
            );
        } else {
            for dev in self.connection.available_devices.iter() {
                let name = crate::core::device::get_profile(dev.vendor_id, dev.product_id)
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

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> =
            if self.connection.status == ConnectionStatus::Disconnected {
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
                            .padding(SPACE_16)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into(),
                        LayoutBucket::Wide => self.view_wide(),
                    }
                })
                .into()
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
        use iced::keyboard;
        use iced::time;

        use std::time::Duration;
        async fn tick() -> Message {
            Message::Tick(std::time::Instant::now())
        }
        let tick_sub = time::repeat(|| Box::pin(tick()), Duration::from_secs(2));
        let close_sub = iced::window::close_requests().map(Message::CloseRequested);
        let keyboard_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                if modifiers.control() {
                    if modifiers.shift() && key == keyboard::Key::Character("Z".into()) {
                        return Some(Message::Editor(EditorMessage::Redo));
                    }
                    if key == keyboard::Key::Character("z".into()) {
                        return Some(Message::Editor(EditorMessage::Undo));
                    }
                    if key == keyboard::Key::Character("s".into()) {
                        return Some(Message::Profiles(ProfilesMessage::SaveProfilePressed));
                    }
                    if key == keyboard::Key::Character("r".into()) {
                        return Some(Message::Editor(EditorMessage::PullPressed));
                    }
                    if modifiers.shift() && key == keyboard::Key::Character("r".into()) {
                        return Some(Message::Editor(EditorMessage::ResetFiltersPressed));
                    }
                    if key == keyboard::Key::Named(keyboard::key::Named::Enter) {
                        return Some(Message::Editor(EditorMessage::PushPressed));
                    }
                    if key == keyboard::Key::Character("v".into()) {
                        return Some(Message::AutoEq(AutoEqMessage::ImportFromClipboard));
                    }
                }
                if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                    return Some(Message::DismissConfirmDialog);
                }
            }
            None
        });
        let file_drop_sub = if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Subscription::none()
        } else {
            iced::event::listen_with(|event, _status, _id| {
                if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
                    Some(Message::Profiles(ProfilesMessage::FileImported(Some(path))))
                } else {
                    None
                }
            })
        };
        Subscription::batch(vec![tick_sub, close_sub, keyboard_sub, file_drop_sub])
    }
}

pub fn run() -> iced::Result {
    let mut window_settings = iced::window::Settings {
        min_size: Some(iced::Size::new(
            LAYOUT_WINDOW_MIN_WIDTH,
            LAYOUT_WINDOW_MIN_HEIGHT,
        )),
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = "frost-tune".to_string();
    }

    iced::application(MainWindow::new, MainWindow::update, MainWindow::view)
        .title(MainWindow::title)
        .subscription(MainWindow::subscription)
        .theme(MainWindow::app_theme)
        .window(window_settings)
        .run()
}

pub fn run_with_diagnostics(recent_logs: Vec<String>) -> iced::Result {
    let events: Vec<DiagnosticEvent> = recent_logs
        .into_iter()
        .map(|line| {
            if let Some(event) = parse_diagnostic_log_line(&line) {
                event
            } else {
                DiagnosticEvent::new(LogLevel::Info, Source::UI, line)
            }
        })
        .collect();
    let diagnostics = DiagnosticsStore::from_events(events);
    let mut window_settings = iced::window::Settings {
        min_size: Some(iced::Size::new(
            LAYOUT_WINDOW_MIN_WIDTH,
            LAYOUT_WINDOW_MIN_HEIGHT,
        )),
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = "frost-tune".to_string();
    }

    iced::application(
        move || MainWindow::new_with_diagnostics(diagnostics.clone()),
        MainWindow::update,
        MainWindow::view,
    )
    .title(MainWindow::title)
    .subscription(MainWindow::subscription)
    .theme(MainWindow::app_theme)
    .window(window_settings)
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
        // Testing out range values
        assert_eq!(parse_freq_string("21k"), None);
        assert_eq!(parse_freq_string("10"), None);
        // Invalid strings
        assert_eq!(parse_freq_string(""), None);
        assert_eq!(parse_freq_string("abc"), None);
        assert_eq!(parse_freq_string("k"), None);
    }
}
