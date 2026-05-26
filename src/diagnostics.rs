// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use serde::Serialize;
use std::collections::VecDeque;

const MAX_EVENTS: usize = 500;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Source {
    UI,
    Worker,
    HID,
    AutoEQ,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::UI => write!(f, "UI"),
            Source::Worker => write!(f, "Worker"),
            Source::HID => write!(f, "HID"),
            Source::AutoEQ => write!(f, "AutoEQ"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticEvent {
    pub timestamp: String,
    pub level: LogLevel,
    pub source: Source,
    pub message: String,
    pub context: Option<std::collections::HashMap<String, String>>,
}

impl DiagnosticEvent {
    pub fn new(level: LogLevel, source: Source, message: impl Into<String>) -> Self {
        let timestamp = chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();
        DiagnosticEvent {
            timestamp,
            level,
            source,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let context = self
            .context
            .get_or_insert_with(std::collections::HashMap::new);
        context.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticsStore {
    events: VecDeque<DiagnosticEvent>,
}

impl DiagnosticsStore {
    pub fn push(&mut self, event: DiagnosticEvent) {
        if let Some(last) = self.events.back() {
            if last.level == event.level
                && last.source == event.source
                && last.message == event.message
            {
                return;
            }
        }

        let line = format_diagnostic_log_line(&event);
        let _ = crate::storage::append_diagnostics_log(&line);

        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn events(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.events.iter()
    }

    pub fn from_events(events: Vec<DiagnosticEvent>) -> Self {
        let mut store = DiagnosticsStore::default();
        let mut queue: VecDeque<DiagnosticEvent> = events.into_iter().collect();
        while queue.len() > MAX_EVENTS {
            queue.pop_front();
        }
        store.events = queue;
        store
    }

    pub fn errors(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.events
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn count(&self) -> usize {
        self.events.len()
    }
}

pub fn format_diagnostics(
    store: &DiagnosticsStore,
    app_version: &str,
    connection_status: &str,
) -> String {
    let mut output = Vec::new();

    output.push("=== Frost-Tune Diagnostics ===".to_string());
    output.push(format!("Version: {}", app_version));
    output.push(format!(
        "Timestamp: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    output.push(format!("Connection Status: {}", connection_status));
    output.push("".to_string());
    output.push("=== Events ===".to_string());

    for event in store.events() {
        let level_str = format!("[{}]", event.level);
        let source_str = format!("[{}]", event.source);
        output.push(format!(
            "{} {} {} {}",
            event.timestamp, level_str, source_str, event.message
        ));

        if let Some(ref ctx) = event.context {
            for (k, v) in ctx.iter() {
                output.push(format!("  {}: {}", k, v));
            }
        }
    }

    output.push("".to_string());
    output.push(format!("=== Total Events: {} ===", store.count()));

    output.join("\n")
}

pub fn format_diagnostic_log_line(event: &DiagnosticEvent) -> String {
    let mut line = format!(
        "{} [{}] [{}] {}",
        event.timestamp, event.level, event.source, event.message
    );
    if let Some(ref ctx) = event.context {
        for (k, v) in ctx {
            line.push_str(&format!(" | {}={}", k, v));
        }
    }
    line
}

pub fn parse_diagnostic_log_line(line: &str) -> Option<DiagnosticEvent> {
    let line = line.trim();
    let mut parts = line.splitn(5, ' ');
    let date = parts.next()?;
    let time = parts.next()?;
    let timestamp = format!("{} {}", date, time);
    let level_raw = parts.next()?;
    let source_raw = parts.next()?;
    let message = parts.next()?.to_string();
    let message = message.split(" | ").next().unwrap_or(&message).to_string();

    let level = match level_raw.trim_matches(&['[', ']'][..]) {
        "INFO" => LogLevel::Info,
        "WARN" => LogLevel::Warn,
        "ERROR" => LogLevel::Error,
        _ => return None,
    };

    let source = match source_raw.trim_matches(&['[', ']'][..]) {
        "UI" => Source::UI,
        "Worker" => Source::Worker,
        "HID" => Source::HID,
        "AutoEQ" => Source::AutoEQ,
        _ => return None,
    };

    Some(DiagnosticEvent {
        timestamp,
        level,
        source,
        message,
        context: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_event_creation() {
        let event = DiagnosticEvent::new(LogLevel::Info, Source::UI, "Test info message");
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.source, Source::UI);
        assert_eq!(event.message, "Test info message");
        assert!(event.context.is_none());

        let event_with_ctx = event
            .with_context("key1", "val1")
            .with_context("key2", "val2");
        let ctx = event_with_ctx.context.as_ref().unwrap();
        assert_eq!(ctx.get("key1").unwrap(), "val1");
        assert_eq!(ctx.get("key2").unwrap(), "val2");
    }

    #[test]
    fn test_diagnostics_store_deduplication() {
        let mut store = DiagnosticsStore::default();
        let event1 = DiagnosticEvent::new(LogLevel::Info, Source::UI, "Message 1");
        let event2 = DiagnosticEvent::new(LogLevel::Info, Source::UI, "Message 1");
        let event3 = DiagnosticEvent::new(LogLevel::Info, Source::UI, "Message 2");

        store.push(event1);
        store.push(event2); // Should be deduplicated as it is consecutively identical
        assert_eq!(store.count(), 1);

        store.push(event3);
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_diagnostics_store_capping() {
        let mut store = DiagnosticsStore::default();
        for i in 0..600 {
            let event = DiagnosticEvent::new(LogLevel::Info, Source::UI, format!("Message {}", i));
            store.push(event);
        }
        assert_eq!(store.count(), MAX_EVENTS);
    }

    #[test]
    fn test_diagnostics_store_errors_filtering() {
        let mut store = DiagnosticsStore::default();
        store.push(DiagnosticEvent::new(
            LogLevel::Info,
            Source::UI,
            "Info message",
        ));
        store.push(DiagnosticEvent::new(
            LogLevel::Error,
            Source::Worker,
            "Error message",
        ));
        store.push(DiagnosticEvent::new(
            LogLevel::Warn,
            Source::HID,
            "Warn message",
        ));

        let errors: Vec<&DiagnosticEvent> = store.errors().collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error message");
    }

    #[test]
    fn test_diagnostics_store_from_events() {
        let mut events = Vec::new();
        for i in 0..600 {
            events.push(DiagnosticEvent::new(
                LogLevel::Info,
                Source::UI,
                format!("Msg {}", i),
            ));
        }
        let store = DiagnosticsStore::from_events(events);
        assert_eq!(store.count(), MAX_EVENTS);
    }

    #[test]
    fn test_format_diagnostics() {
        let mut store = DiagnosticsStore::default();
        store.push(DiagnosticEvent::new(
            LogLevel::Info,
            Source::UI,
            "Diagnostics started",
        ));
        let output = format_diagnostics(&store, "1.0.0", "Connected");
        assert!(output.contains("=== Frost-Tune Diagnostics ==="));
        assert!(output.contains("Version: 1.0.0"));
        assert!(output.contains("Connection Status: Connected"));
        assert!(output.contains("Diagnostics started"));
    }

    #[test]
    fn test_diagnostic_log_line_roundtrip() {
        let event = DiagnosticEvent::new(LogLevel::Warn, Source::HID, "Device disconnected")
            .with_context("device_id", "1234");
        let formatted = format_diagnostic_log_line(&event);
        let parsed = parse_diagnostic_log_line(&formatted).unwrap();

        assert_eq!(parsed.level, LogLevel::Warn);
        assert_eq!(parsed.source, Source::HID);
        assert_eq!(parsed.message, "Device disconnected");
    }
}
