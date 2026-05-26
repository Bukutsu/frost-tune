// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind, Result};
use std::fs;
use std::path::PathBuf;

fn ensure_dir(path: &PathBuf, kind: ErrorKind, message: &str) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| AppError::new(kind, format!("{}: {}", message, e)))?;
    }
    Ok(())
}

pub(crate) fn get_base_dir() -> Result<PathBuf> {
    let env_home = if nix::unistd::Uid::current().is_root() {
        None
    } else {
        std::env::var("FROST_TUNE_HOME").ok()
    };

    let base_dir = match env_home {
        Some(val) => PathBuf::from(val),
        None => dirs::data_dir()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::StorageError,
                    "Failed to get standard data directory. Set FROST_TUNE_HOME to override.",
                )
            })?
            .join("frost-tune"),
    };

    ensure_dir(
        &base_dir,
        ErrorKind::StorageError,
        "Failed to create application data directory",
    )?;

    Ok(base_dir)
}

pub(crate) fn get_profiles_dir() -> Result<PathBuf> {
    let profiles_dir = get_base_dir()?.join("profiles");
    ensure_dir(
        &profiles_dir,
        ErrorKind::StorageError,
        "Failed to create profiles directory",
    )?;
    Ok(profiles_dir)
}

pub fn get_diagnostics_log_path() -> Result<PathBuf> {
    Ok(get_base_dir()?.join("diagnostics.log"))
}

pub fn get_profiles_dir_mtime() -> Option<std::time::SystemTime> {
    get_profiles_dir()
        .ok()
        .and_then(|dir| fs::metadata(dir).ok().and_then(|m| m.modified().ok()))
}

pub fn get_profiles_dir_display() -> String {
    get_profiles_dir()
        .map(|p| {
            if let Some(home) = dirs::home_dir() {
                if let Ok(stripped) = p.strip_prefix(&home) {
                    return format!("~{}{}", std::path::MAIN_SEPARATOR, stripped.display());
                }
            }
            p.to_string_lossy().into_owned()
        })
        .unwrap_or_else(|_| "~/.local/share/frost-tune/profiles".to_string())
}

pub(crate) fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect()
}
