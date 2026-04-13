use serde::Serialize;
use std::collections::VecDeque;

const MAX_EVENTS: usize = 500;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        DiagnosticEvent {
            timestamp,
            level,
            source,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let context = self.context.get_or_insert_with(std::collections::HashMap::new);
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
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn events(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.events.iter()
    }

    pub fn errors(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.events.iter().filter(|e| matches!(e.level, LogLevel::Error))
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn count(&self) -> usize {
        self.events.len()
    }
}

pub fn format_diagnostics(store: &DiagnosticsStore, app_version: &str, connection_status: &str) -> String {
    let mut output = Vec::new();
    
    output.push("=== Frost-Tune Diagnostics ===".to_string());
    output.push(format!("Version: {}", app_version));
    output.push(format!("Timestamp: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    output.push(format!("Connection Status: {}", connection_status));
    output.push("".to_string());
    output.push("=== Events ===".to_string());
    
    for event in store.events() {
        let level_str = format!("[{}]", event.level);
        let source_str = format!("[{}]", event.source);
        output.push(format!("{} {} {} {}", event.timestamp, level_str, source_str, event.message));
        
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