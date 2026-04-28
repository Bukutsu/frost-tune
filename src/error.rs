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

    /// Heuristic-based error classification from string messages.
    /// This matches substrings and is potentially fragile if upstream errors change wording.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_from_string() {
        assert_eq!(ErrorKind::from_string("Device not found on USB bus"), ErrorKind::NotConnected);
        assert_eq!(ErrorKind::from_string("POLKIT_AUTH_REQUIRED"), ErrorKind::PolkitAuthRequired);
        assert_eq!(ErrorKind::from_string("Permission denied (os error 13)"), ErrorKind::PermissionDenied);
        assert_eq!(ErrorKind::from_string("Device is busy"), ErrorKind::DeviceBusy);
        assert_eq!(ErrorKind::from_string("USB timeout reading from endpoint"), ErrorKind::ReadTimeout);
        assert_eq!(ErrorKind::from_string("Verification mismatch at byte 5"), ErrorKind::VerifyFailed);
        assert_eq!(ErrorKind::from_string("Failed to parse filter packet"), ErrorKind::ParseError);
        assert_eq!(ErrorKind::from_string("Failed to load profiles"), ErrorKind::StorageError);
        assert_eq!(ErrorKind::from_string("Some unknown error occurred"), ErrorKind::Unknown);
    }
}
