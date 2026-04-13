use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppError(String);

impl AppError {
    pub fn new(msg: impl Into<String>) -> Self {
        AppError(msg.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError(s.to_string())
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.0
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotConnected,
    PermissionDenied,
    DeviceBusy,
    ReadTimeout,
    WriteError,
    VerifyFailed,
    RollbackFailed,
    DeviceLost,
    Unknown,
}

impl ErrorKind {
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorKind::NotConnected => "Device not found. Is it plugged in?",
            ErrorKind::PermissionDenied => "Access denied. Check USB permissions.",
            ErrorKind::DeviceBusy => "Device is busy. Another app may be connected.",
            ErrorKind::ReadTimeout => "USB read timeout. Try again.",
            ErrorKind::WriteError => "USB write failed.",
            ErrorKind::VerifyFailed => "Verification failed. Changes not applied.",
            ErrorKind::RollbackFailed => "Failed to restore previous settings. Device may be in an inconsistent state.",
            ErrorKind::DeviceLost => "Device disconnected during operation.",
            ErrorKind::Unknown => "Unknown error.",
        }
    }
    
    pub fn from_string(s: &str) -> Self {
        if s.contains("Not connected") || s.contains("not found") || s.contains("No such") {
            ErrorKind::NotConnected
        } else if s.contains("Permission denied") || s.contains("Access denied") {
            ErrorKind::PermissionDenied
        } else if s.contains("busy") || s.contains("in use") {
            ErrorKind::DeviceBusy
        } else if s.contains("timeout") || s.contains("Timeout") {
            ErrorKind::ReadTimeout
        } else if s.contains("verification") || s.contains("mismatch") || s.contains("Verify") {
            ErrorKind::VerifyFailed
        } else if s.contains("rollback") || s.contains("restore") {
            ErrorKind::RollbackFailed
        } else if s.contains("failed") || s.contains("error") {
            ErrorKind::Unknown
        } else {
            ErrorKind::Unknown
        }
    }
}

pub const DEVICE_NOT_FOUND: &str = "Device not found. Is it plugged in?";
pub const PERMISSION_DENIED: &str = "Access denied. Check USB permissions.";
pub const DEVICE_BUSY: &str = "Device is busy.";
pub const READ_TIMEOUT: &str = "USB read timeout.";
pub const WRITE_ERROR: &str = "USB write failed.";
pub const PARSE_ERROR: &str = "Failed to parse filter data.";
pub const VERIFY_FAILED: &str = "Verification failed. Changes not applied.";