use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[error("{message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
}

impl AppError {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        AppError {
            kind,
            message: msg.into(),
            context: None,
        }
    }

    pub fn general(msg: impl Into<String>) -> Self {
        AppError::new(ErrorKind::Unknown, msg)
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn user_message(&self) -> &'static str {
        self.kind.user_message()
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::general(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::general(s)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    NotConnected,
    PermissionDenied,
    PolkitAuthRequired,
    DeviceBusy,
    ReadTimeout,
    WriteError,
    VerifyFailed,
    RollbackFailed,
    DeviceLost,
    HardwareError,
    ParseError,
    StorageError,
    IpcError,
    Unknown,
}

impl ErrorKind {
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorKind::NotConnected => "Device not found. Is it plugged in?",
            ErrorKind::PermissionDenied => "Access denied. Check USB permissions.",
            ErrorKind::PolkitAuthRequired => {
                "Authentication required to access USB DAC on Linux. Approve the polkit prompt."
            }
            ErrorKind::DeviceBusy => "Device is busy. Another app may be connected.",
            ErrorKind::ReadTimeout => "USB read timeout. Try again.",
            ErrorKind::WriteError => "USB write failed.",
            ErrorKind::VerifyFailed => "Verification failed. Changes not applied.",
            ErrorKind::RollbackFailed => {
                "Failed to restore previous settings. Device may be in an inconsistent state."
            }
            ErrorKind::DeviceLost => "Device disconnected during operation.",
            ErrorKind::HardwareError => "Hardware protocol error.",
            ErrorKind::ParseError => "Failed to parse data.",
            ErrorKind::StorageError => "Profile storage error.",
            ErrorKind::IpcError => "IPC communication error with background helper.",
            ErrorKind::Unknown => "Unknown error.",
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
