// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::autoeq;
use crate::error::{AppError, ErrorKind, Result};
use crate::models::PEQData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub data: PEQData,
    pub modified: Option<String>,
}

fn get_base_dir() -> Result<PathBuf> {
    let base_dir = dirs::data_dir()
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::StorageError,
                "Failed to get standard data directory",
            )
        })?
        .join("frost-tune");

    if !base_dir.exists() {
        fs::create_dir_all(&base_dir).map_err(|e| {
            AppError::new(
                ErrorKind::StorageError,
                format!("Failed to create application data directory: {}", e),
            )
        })?;
    }

    Ok(base_dir)
}

fn get_profiles_dir() -> Result<PathBuf> {
    let profiles_dir = get_base_dir()?.join("profiles");

    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir).map_err(|e| {
            AppError::new(
                ErrorKind::StorageError,
                format!("Failed to create profiles directory: {}", e),
            )
        })?;
    }

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
            let path_str = p.to_string_lossy().into_owned();
            if let Some(home) = dirs::home_dir() {
                let home_str = home.to_string_lossy();
                if path_str.starts_with(home_str.as_ref()) {
                    return path_str.replacen(home_str.as_ref(), "~", 1);
                }
            }
            path_str
        })
        .unwrap_or_else(|_| "~/.local/share/frost-tune/profiles".to_string())
}

pub fn load_all_profiles() -> Result<(Vec<Profile>, Vec<String>)> {
    let dir = get_profiles_dir()?;
    let mut profiles = Vec::new();
    let mut errors = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to read profiles directory: {}", e),
        )
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "txt") {
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.len() > 1024 * 1024 {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        errors.push(format!("Profile '{}' is too large (> 1MB)", name));
                    }
                    continue;
                }
            }

            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("Cannot read profile '{}': {}", name, e));
                        continue;
                    }
                };
                match autoeq::parse_autoeq_text(&content) {
                    Ok((data, warnings)) => {
                        if !warnings.is_empty() {
                            for w in &warnings {
                                log::warn!("Profile {} warning: {}", name, w);
                            }
                        }
                        let modified = fs::metadata(&path)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .map(|t| {
                                chrono::DateTime::<chrono::Local>::from(t)
                                    .format("%Y-%m-%d %H:%M")
                                    .to_string()
                            });
                        profiles.push(Profile {
                            name: name.to_string(),
                            data,
                            modified,
                        });
                    }
                    Err(e) => {
                        log::warn!("Failed to parse profile {}: {}", name, e);
                        errors.push(format!("Profile '{}' failed to parse: {}", name, e));
                    }
                }
            }
        }
    }

    // Sort profiles by name
    profiles.sort_by_key(|a| a.name.to_lowercase());

    Ok((profiles, errors))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect()
}

pub fn save_profile(name: &str, data: &PEQData) -> Result<()> {
    let dir = get_profiles_dir()?;

    let sanitized_name = sanitize_name(name);

    if sanitized_name.is_empty() {
        return Err(AppError::new(
            ErrorKind::StorageError,
            "Invalid profile name",
        ));
    }

    let filename = format!("{}.txt", sanitized_name);
    let path = dir.join(&filename);
    let tmp_path = dir.join(format!(".{}.tmp", sanitized_name));

    let content = autoeq::peq_to_autoeq(data);
    fs::write(&tmp_path, &content).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to write temp profile: {}", e),
        )
    })?;
    fs::rename(&tmp_path, &path).map_err(|e| {
        if let Err(cleanup_err) = fs::remove_file(&tmp_path) {
            log::warn!(
                "Failed to clean up temp file after rename error: {}",
                cleanup_err
            );
        }
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to finalize profile save: {}", e),
        )
    })?;

    if let Ok(dir_file) = fs::File::open(&dir) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}

pub fn delete_profile(name: &str) -> Result<()> {
    let dir = get_profiles_dir()?;
    let sanitized_name = sanitize_name(name);
    let filename = format!("{}.txt", sanitized_name);
    let path = dir.join(filename);

    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            AppError::new(
                ErrorKind::StorageError,
                format!("Failed to delete profile: {}", e),
            )
        })?;
    }

    Ok(())
}

pub fn import_profile(path: &std::path::Path) -> Result<Profile> {
    let metadata = fs::metadata(path).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to stat profile file: {}", e),
        )
    })?;

    if metadata.len() > 1024 * 1024 {
        return Err(AppError::new(
            ErrorKind::StorageError,
            "Profile file is too large (> 1MB), refusing to read.",
        ));
    }

    let content = fs::read_to_string(path).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to read profile file: {}", e),
        )
    })?;
    let (data, warnings) = autoeq::parse_autoeq_text(&content).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to parse profile: {}", e),
        )
    })?;
    if !warnings.is_empty() {
        log::warn!("Import warnings for profile: {:?}", warnings);
    }
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Profile")
        .to_string();
    Ok(Profile {
        name,
        data,
        modified: None,
    })
}

pub fn export_profile(path: &std::path::Path, data: &PEQData) -> Result<()> {
    let content = autoeq::peq_to_autoeq(data);
    fs::write(path, content).map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to write profile file: {}", e),
        )
    })
}

pub fn open_profiles_dir() -> Result<()> {
    let dir = get_profiles_dir()?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| {
                AppError::new(
                    ErrorKind::StorageError,
                    format!("Failed to open explorer: {}", e),
                )
            })?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| {
                AppError::new(
                    ErrorKind::StorageError,
                    format!("Failed to open folder: {}", e),
                )
            })?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| {
                AppError::new(
                    ErrorKind::StorageError,
                    format!("Failed to open file manager: {}", e),
                )
            })?;
    }

    Ok(())
}

pub fn append_diagnostics_log(line: &str) -> Result<()> {
    let path = get_diagnostics_log_path()?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| {
            AppError::new(
                ErrorKind::StorageError,
                format!("Failed to open diagnostics log: {}", e),
            )
        })?;
    use std::io::Write;
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

    let content = std::fs::read_to_string(&path).map_err(|e| {
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
            std::fs::write(&tmp, lines.join("\n") + "\n").and_then(|_| std::fs::rename(&tmp, &path))
        {
            log::warn!("Failed to truncate diagnostics log: {}", e);
            let _ = std::fs::remove_file(&tmp);
        }
    } else if lines.len() > limit {
        lines = lines.split_off(lines.len() - limit);
    }
    Ok(lines)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Settings {
    pub auto_pull_on_connect: bool,
}

pub fn load_settings() -> Settings {
    get_base_dir()
        .map(|base| base.join("settings.json"))
        .and_then(|path| {
            if path.exists() {
                let content = fs::read_to_string(path).map_err(|e| {
                    AppError::new(
                        ErrorKind::StorageError,
                        format!("Failed to read settings file: {}", e),
                    )
                })?;
                let settings = serde_json::from_str(&content).map_err(|e| {
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
        .unwrap_or_else(|_| Settings::default())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autoeq;
    use crate::models::{Filter, FilterType, PEQData};

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hello World"), "Hello World");
        assert_eq!(sanitize_name("Profile_1-2"), "Profile_1-2");
        assert_eq!(sanitize_name("Bad@Name#$"), "BadName");
        assert_eq!(sanitize_name(""), "");
        assert_eq!(sanitize_name("Normal Name 123"), "Normal Name 123");
    }

    #[test]
    fn test_save_and_load_profile_roundtrip() {
        let test_name = "___test_roundtrip_profile___";
        let data = PEQData {
            filters: vec![Filter {
                index: 0,
                enabled: true,
                filter_type: FilterType::Peak,
                freq: 1000,
                gain: 5.0,
                q: 1.5,
            }],
            global_gain: -2,
        };

        let save_result = save_profile(test_name, &data);
        assert!(save_result.is_ok(), "Saving profile should succeed");

        let load_result = load_all_profiles();
        assert!(load_result.is_ok());
        let (profiles, _) = load_result.unwrap();

        let found = profiles.iter().any(|p| p.name == test_name);
        assert!(found, "Test profile should be in loaded profiles");

        let _ = delete_profile(test_name);
    }

    #[test]
    fn test_delete_profile() {
        let test_name = "___test_delete_profile___";
        let data = PEQData {
            filters: vec![Filter::enabled(0, true)],
            global_gain: 0,
        };

        let _ = save_profile(test_name, &data);

        let delete_result = delete_profile(test_name);
        assert!(delete_result.is_ok(), "Deleting profile should succeed");

        let (profiles, _) = load_all_profiles().unwrap_or_default();
        let found = profiles.iter().any(|p| p.name == test_name);
        assert!(!found, "Deleted profile should not be in loaded profiles");
    }

    #[test]
    fn test_import_profile() {
        let content = "Preamp: -3 dB\nFilter 1: ON PK Fc 1000 Hz Gain 5.0 dB Q 1.0";
        let (data, warnings) = autoeq::parse_autoeq_text(content).unwrap();

        assert_eq!(data.global_gain, -3);
        assert!(data.filters[0].enabled);
        assert_eq!(data.filters[0].freq, 1000);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_export_profile_format() {
        let data = PEQData {
            filters: vec![Filter::enabled(0, true)],
            global_gain: 0,
        };

        let output = autoeq::peq_to_autoeq(&data);
        assert!(output.contains("Preamp: 0 dB"));
        assert!(output.contains("Filter 1: ON"));
    }

    #[test]
    fn test_append_and_load_diagnostics() {
        let test_line = r#"{"level":"INFO","source":"UI","message":"Test log entry"}"#;

        let append_result = append_diagnostics_log(test_line);
        assert!(
            append_result.is_ok(),
            "Appending to diagnostics log should succeed"
        );

        let loaded = load_recent_diagnostics(10);
        assert!(loaded.is_ok());
        let logs = loaded.unwrap();
        assert!(!logs.is_empty(), "Should have loaded some diagnostics");
    }

    #[test]
    fn test_load_profiles_error_handling() {
        let result = load_all_profiles();
        assert!(
            result.is_ok(),
            "Loading profiles should not error even with invalid files"
        );
        let (_, errors) = result.unwrap();
        for error in errors {
            println!("Profile parse error (expected in test): {}", error);
        }
    }
}
