// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::Filter;
use crate::diagnostics::{
    parse_diagnostic_log_line, DiagnosticEvent, DiagnosticsStore, LogLevel, Source,
};
use crate::hardware::worker::UsbWorker;
use crate::ui::components::connection::{ConnectionComponent, ConnectionStatus};
use crate::ui::components::editor::{EditorComponent, EditorData, EditorUI};
use crate::ui::messages::{Message, ProfilesMessage, StatusMessage, StatusSeverity};
use crate::ui::state::AppState;
use crate::ui::theme;
use crate::ui::tokens::{LAYOUT_WINDOW_MIN_HEIGHT, LAYOUT_WINDOW_MIN_WIDTH};
use iced::{Element, Subscription, Task};
use std::sync::Arc;

pub const STATUS_AUTO_CLEAR_SECS: u64 = 5;
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

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

impl AppState {
    fn new_with_diagnostics(diagnostics: DiagnosticsStore) -> (Self, Task<Message>) {
        let worker = Arc::new(UsbWorker::new());
        let default_filters: Vec<Filter> =
            (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let settings = crate::storage::load_settings();
        let window = AppState {
            editor: EditorComponent {
                data: EditorData {
                    filters: default_filters,
                    ..Default::default()
                },
                ui: EditorUI {
                    snap_to_iso_enabled: true,
                    auto_pull_on_connect: settings.auto_pull_on_connect,
                    skip_push_verification: settings.skip_push_verification,
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
            iced::font::load(crate::ui::tokens::ICON_FONT_BYTES).map(|_| Message::NoOp);

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

    /// Sets the status banner and echoes the message to the diagnostics log.
    pub fn set_status(
        &mut self,
        content: impl Into<String>,
        severity: StatusSeverity,
    ) -> Task<Message> {
        self.set_status_impl(content, severity, true)
    }

    /// Sets the status banner without echoing to the diagnostics log.
    /// Use this for profile operations that push their own diagnostics events.
    pub fn set_status_silent(
        &mut self,
        content: impl Into<String>,
        severity: StatusSeverity,
    ) -> Task<Message> {
        self.set_status_impl(content, severity, false)
    }

    fn set_status_impl(
        &mut self,
        content: impl Into<String>,
        severity: StatusSeverity,
        echo_diagnostics: bool,
    ) -> Task<Message> {
        let content = content.into();
        if echo_diagnostics {
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

    fn view(&self) -> Element<'_, Message> {
        crate::ui::layout::view(self)
    }

    fn subscription(&self) -> Subscription<Message> {
        crate::ui::subscriptions::subscription(self)
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

    iced::application(AppState::new, AppState::update, AppState::view)
        .title(AppState::title)
        .subscription(AppState::subscription)
        .theme(AppState::app_theme)
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
        move || AppState::new_with_diagnostics(diagnostics.clone()),
        AppState::update,
        AppState::view,
    )
    .title(AppState::title)
    .subscription(AppState::subscription)
    .theme(AppState::app_theme)
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
