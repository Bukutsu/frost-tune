// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

#[cfg(target_os = "linux")]
use crate::error::Result;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};
#[cfg(target_os = "linux")]
use iced::futures::future::BoxFuture;

/// A trait representing a communication channel with an elevated helper process.
/// This allows different transport implementations across operating systems (e.g. Linux pkexec, macOS helper).
#[cfg(target_os = "linux")]
pub trait Transport: Send + Sync {
    /// Perform a single request-response exchange with the helper.
    fn round_trip<'a>(
        &'a self,
        request: &'a HelperRequest,
    ) -> BoxFuture<'a, Result<HelperResponse>>;

    /// Gracefully shutdown the transport channel.
    fn shutdown(&mut self);
}
