// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Local,
    #[cfg(target_os = "linux")]
    Elevated,
}
