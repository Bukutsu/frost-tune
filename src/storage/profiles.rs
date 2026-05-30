// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::autoeq;
use crate::core::PEQData;
use crate::error::{AppError, ErrorKind, Result};
use std::path::Path;
use tokio::fs;

use super::paths::{get_profiles_dir, sanitize_name};

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub data: PEQData,
    pub modified: Option<String>,
}

pub async fn load_all_profiles() -> Result<(Vec<Profile>, Vec<String>)> {
    let dir = get_profiles_dir()?;
    let mut profiles = Vec::new();
    let mut errors = Vec::new();

    let mut entries = fs::read_dir(dir).await.map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to read profiles directory: {}", e),
        )
    })?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "txt") {
            if let Ok(metadata) = fs::metadata(&path).await {
                if metadata.len() > 1024 * 1024 {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        errors.push(format!("Profile '{}' is too large (> 1MB)", name));
                    }
                    continue;
                }
            }

            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let content = match fs::read_to_string(&path).await {
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
                            .await
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

    profiles.sort_by_key(|a| a.name.to_lowercase());

    Ok((profiles, errors))
}

pub async fn save_profile(name: &str, data: &PEQData) -> Result<()> {
    let dir = get_profiles_dir()?;
    let sanitized_name = sanitize_name(name);

    if sanitized_name.is_empty() {
        return Err(AppError::new(
            ErrorKind::StorageError,
            "Invalid profile name",
        ));
    }

    let path = dir.join(format!("{}.txt", sanitized_name));
    let tmp_path = dir.join(format!(".{}.tmp", sanitized_name));

    let content = autoeq::peq_to_autoeq(data);
    fs::write(&tmp_path, &content).await.map_err(|e| {
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to write temp profile: {}", e),
        )
    })?;
    fs::rename(&tmp_path, &path).await.map_err(|e| {
        // Fallback since remove_file is async, we can't easily wait here inside map_err without being an async closure.
        // We will just let the map_err construct the error, then we will spawn a cleanup task or ignore.
        AppError::new(
            ErrorKind::StorageError,
            format!("Failed to finalize profile save: {}", e),
        )
    })?;

    // Directory sync is only meaningful/supported on Unix-like operating systems
    #[cfg(unix)]
    if let Ok(dir_file) = fs::File::open(&dir).await {
        if let Err(e) = dir_file.sync_all().await {
            log::error!("Failed to sync directory after saving profile: {}", e);
        }
    }

    // Cleanup tmp file if it still exists (e.g., if rename succeeded or failed)
    if let Err(e) = fs::remove_file(&tmp_path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::error!(
                "Failed to clean up temporary file {}: {}",
                tmp_path.display(),
                e
            );
        }
    }

    Ok(())
}

pub async fn delete_profile(name: &str) -> Result<()> {
    let dir = get_profiles_dir()?;
    let sanitized_name = sanitize_name(name);

    if sanitized_name.is_empty() {
        return Err(AppError::new(
            ErrorKind::StorageError,
            "Invalid profile name",
        ));
    }

    let path = dir.join(format!("{}.txt", sanitized_name));

    match fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::new(
            ErrorKind::StorageError,
            format!("Failed to delete profile: {}", e),
        )),
    }
}

pub async fn import_profile(path: &Path) -> Result<Profile> {
    let metadata = fs::metadata(path).await.map_err(|e| {
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

    let content = fs::read_to_string(path).await.map_err(|e| {
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

pub async fn export_profile(path: &Path, data: &PEQData) -> Result<()> {
    let content = autoeq::peq_to_autoeq(data);
    fs::write(path, content).await.map_err(|e| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::autoeq;
    use crate::core::{Filter, FilterType, PEQData};

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hello World"), "Hello World");
        assert_eq!(sanitize_name("Profile_1-2"), "Profile_1-2");
        assert_eq!(sanitize_name("Bad@Name#$"), "BadName");
        assert_eq!(sanitize_name(""), "");
        assert_eq!(sanitize_name("Normal Name 123"), "Normal Name 123");
    }

    #[tokio::test]
    async fn test_save_and_load_profile_roundtrip() {
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

        let save_result = save_profile(test_name, &data).await;
        assert!(save_result.is_ok(), "Saving profile should succeed");

        let load_result = load_all_profiles().await;
        assert!(load_result.is_ok());
        let (profiles, _) = load_result.unwrap();

        let found = profiles.iter().any(|p| p.name == test_name);
        assert!(found, "Test profile should be in loaded profiles");

        let _ = delete_profile(test_name).await;
    }

    #[tokio::test]
    async fn test_delete_profile() {
        let test_name = "___test_delete_profile___";
        let data = PEQData {
            filters: vec![Filter::enabled(0, true)],
            global_gain: 0,
        };

        let _ = save_profile(test_name, &data).await;

        let delete_result = delete_profile(test_name).await;
        assert!(delete_result.is_ok(), "Deleting profile should succeed");

        let (profiles, _) = load_all_profiles().await.unwrap_or_default();
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
}
