use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[error("{message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

impl AppError {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        AppError {
            kind,
            message: msg.into(),
        }
    }

    pub fn general(msg: impl Into<String>) -> Self {
        let msg_str = msg.into();
        AppError {
            kind: ErrorKind::from_string(&msg_str),
            message: msg_str,
        }
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
    Unknown,
}

impl ErrorKind {
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorKind::NotConnected => "Device not found. Is it plugged in?",
            ErrorKind::PermissionDenied => "Access denied. Check USB permissions.",
            ErrorKind::PolkitAuthRequired => "Authentication required to access USB DAC on Linux. Approve the polkit prompt.",
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
            ErrorKind::Unknown => "Unknown error.",
        }
    }

    pub fn from_string(s: &str) -> Self {
        if s.contains("Not connected") || s.contains("not found") || s.contains("No such") {
            ErrorKind::NotConnected
        } else if s.contains("POLKIT_AUTH_REQUIRED") || s.contains("Authentication required") {
            ErrorKind::PolkitAuthRequired
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
        } else if s.contains("parse") || s.contains("Parse") {
            ErrorKind::ParseError
        } else if s.contains("storage") || s.contains("Storage") || s.contains("profile") {
            ErrorKind::StorageError
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
