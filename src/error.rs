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

pub const DeviceNotFound: &str = "Device not found. Is it plugged in?";
pub const PermissionDenied: &str = "Access denied. Check USB permissions.";
pub const DeviceBusy: &str = "Device is busy.";
pub const ReadTimeout: &str = "USB read timeout.";
pub const WriteError: &str = "USB write failed.";
pub const ParseError: &str = "Failed to parse filter data.";