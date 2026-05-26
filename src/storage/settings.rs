// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use super::paths::get_base_dir;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Settings {
    pub auto_pull_on_connect: bool,
    pub skip_push_verification: bool,
}

pub fn load_settings() -> Settings {
    get_base_dir()
        .map(|base| base.join("settings.json"))
        .and_then(|path| {
            if path.exists() {
                let content = fs::read_to_string(&path).map_err(|e| {
                    AppError::new(
                        ErrorKind::StorageError,
                        format!("Failed to read settings file: {}", e),
                    )
                })?;
                let settings = serde_json::from_str(&content).map_err(|e| {
                    let bad_path = path.with_extension("json.bad");
                    let _ = fs::rename(&path, &bad_path);
                    log::error!(
                        "Corrupted settings.json detected. Renamed to settings.json.bad. Error: {}",
                        e
                    );
                    AppError::new(
                        ErrorKind::StorageError,
                        format!("Failed to parse settings: {}", e),
                    )
                })?;
                Ok(settings)
            } else {
                Ok(Settings::default())
            }
        })
        .unwrap_or_else(|e| {
            log::warn!("Using default settings due to error: {:?}", e);
            Settings::default()
        })
}

pub fn save_settings(settings: Settings) -> Result<()> {
    let path = get_base_dir()?.join("settings.json");
    let tmp_path = path.with_extension("tmp");
    let content = serde_json::to_string_pretty(&settings).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to serialize settings: {}", e),
        )
    })?;
    fs::write(&tmp_path, content).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to write temporary settings file: {}", e),
        )
    })?;
    fs::rename(&tmp_path, &path).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to rename temporary settings file: {}", e),
        )
    })?;
    Ok(())
}
