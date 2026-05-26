// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

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

    pub fn user_message(&self) -> String {
        self.kind.to_string()
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

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("Device not found. Is it plugged in?")]
    NotConnected,
    #[error("Access denied. Check USB permissions.")]
    PermissionDenied,
    #[error("Authentication required to access USB DAC on Linux. Approve the polkit prompt.")]
    PolkitAuthRequired,
    #[error("Device is busy. Another app may be connected.")]
    DeviceBusy,
    #[error("USB read timeout. Try again.")]
    ReadTimeout,
    #[error("USB write failed.")]
    WriteError,
    #[error("Verification failed. Changes not applied.")]
    VerifyFailed,
    #[error("Failed to restore previous settings. Device may be in an inconsistent state.")]
    RollbackFailed,
    #[error("Device disconnected during operation.")]
    DeviceLost,
    #[error("Hardware protocol error.")]
    HardwareError,
    #[error("Failed to parse data.")]
    ParseError,
    #[error("Profile storage error.")]
    StorageError,
    #[error("IPC communication error with background helper.")]
    IpcError,
    #[error("Invalid or malformed data payload.")]
    InvalidPayload,
    #[error("Operation timed out.")]
    Timeout,
    #[error("Operation cancelled or interrupted.")]
    OperationCancelled,
    #[error("Background worker terminated unexpectedly.")]
    WorkerDied,
    #[error("Unknown error.")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_new() {
        let err = AppError::new(ErrorKind::NotConnected, "test message");
        assert_eq!(err.kind, ErrorKind::NotConnected);
        assert_eq!(err.message, "test message");
        assert_eq!(err.context, None);
    }

    #[test]
    fn test_app_error_general() {
        let err = AppError::general("something went wrong");
        assert_eq!(err.kind, ErrorKind::Unknown);
        assert_eq!(err.message, "something went wrong");
    }

    #[test]
    fn test_app_error_with_context() {
        let err =
            AppError::new(ErrorKind::StorageError, "io error").with_context("failed to write file");
        assert_eq!(err.context, Some("failed to write file".to_string()));
    }

    #[test]
    fn test_app_error_user_message_mapping() {
        let cases = [
            (
                ErrorKind::NotConnected,
                "Device not found. Is it plugged in?",
            ),
            (
                ErrorKind::PermissionDenied,
                "Access denied. Check USB permissions.",
            ),
            (ErrorKind::Timeout, "Operation timed out."),
            (
                ErrorKind::WorkerDied,
                "Background worker terminated unexpectedly.",
            ),
            (ErrorKind::Unknown, "Unknown error."),
        ];
        for (kind, expected) in &cases {
            let err = AppError::new(*kind, "ignored");
            assert_eq!(
                err.user_message().as_str(),
                *expected,
                "mismatch for {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_app_error_from_string() {
        let err: AppError = "custom error".to_string().into();
        assert_eq!(err.kind, ErrorKind::Unknown);
        assert_eq!(err.message, "custom error");
    }

    #[test]
    fn test_app_error_from_str() {
        let err: AppError = "custom error".into();
        assert_eq!(err.kind, ErrorKind::Unknown);
        assert_eq!(err.message, "custom error");
    }

    #[test]
    fn test_error_kind_user_message_all_variants() {
        // Ensure every ErrorKind variant has a non-empty user message
        let variants = [
            ErrorKind::NotConnected,
            ErrorKind::PermissionDenied,
            ErrorKind::PolkitAuthRequired,
            ErrorKind::DeviceBusy,
            ErrorKind::ReadTimeout,
            ErrorKind::WriteError,
            ErrorKind::VerifyFailed,
            ErrorKind::RollbackFailed,
            ErrorKind::DeviceLost,
            ErrorKind::HardwareError,
            ErrorKind::ParseError,
            ErrorKind::StorageError,
            ErrorKind::IpcError,
            ErrorKind::InvalidPayload,
            ErrorKind::Timeout,
            ErrorKind::OperationCancelled,
            ErrorKind::WorkerDied,
            ErrorKind::Unknown,
        ];
        for kind in &variants {
            let msg = kind.to_string();
            assert!(!msg.is_empty(), "user_message for {:?} is empty", kind);
        }
    }

    #[test]
    fn test_result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(AppError::new(ErrorKind::Unknown, "fail"));
        assert!(err.is_err());
    }
}
