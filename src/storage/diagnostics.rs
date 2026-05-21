// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind, Result};
use std::fs;
use std::io::Write;

use super::paths::get_diagnostics_log_path;

pub fn append_diagnostics_log(line: &str) -> Result<()> {
    let path = get_diagnostics_log_path()?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| {
            AppError::new(
                ErrorKind::StorageError,
                format!("Failed to open diagnostics log: {}", e),
            )
        })?;
    file.write_all(line.as_bytes()).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to write diagnostics log: {}", e),
        )
    })?;
    file.write_all(b"\n").map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to finalize diagnostics log: {}", e),
        )
    })?;
    Ok(())
}

pub fn load_recent_diagnostics(limit: usize) -> Result<Vec<String>> {
    let path = get_diagnostics_log_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.len() > 10 * 1024 * 1024 {
            let _ = fs::remove_file(&path);
            return Ok(Vec::new());
        }
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to read diagnostics log: {}", e),
        )
    })?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    const MAX_LOG_LINES: usize = 5000;
    if lines.len() > MAX_LOG_LINES {
        let keep_count = limit.clamp(100, 2000);
        lines = lines.split_off(lines.len() - keep_count);
        let tmp = path.with_extension("log.tmp");
        if let Err(e) =
            fs::write(&tmp, lines.join("\n") + "\n").and_then(|_| fs::rename(&tmp, &path))
        {
            log::warn!("Failed to truncate diagnostics log: {}", e);
            let _ = fs::remove_file(&tmp);
        }
    } else if lines.len() > limit {
        lines = lines.split_off(lines.len() - limit);
    }

    Ok(lines)
}
