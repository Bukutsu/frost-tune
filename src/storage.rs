use crate::autoeq;
use crate::models::PEQData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub data: PEQData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub advanced_filters_expanded: bool,
    pub diagnostics_expanded: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            advanced_filters_expanded: false,
            diagnostics_expanded: false,
        }
    }
}

fn get_profiles_dir() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;
    let profiles_dir = exe_dir.join("profiles");

    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    Ok(profiles_dir)
}

fn get_ui_preferences_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;
    Ok(exe_dir.join("ui_preferences.json"))
}

pub fn load_ui_preferences() -> Result<UiPreferences, String> {
    let path = get_ui_preferences_path()?;
    if !path.exists() {
        return Ok(UiPreferences::default());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read UI preferences: {}", e))?;

    serde_json::from_str::<UiPreferences>(&content)
        .map_err(|e| format!("Failed to parse UI preferences: {}", e))
}

pub fn save_ui_preferences(prefs: &UiPreferences) -> Result<(), String> {
    let path = get_ui_preferences_path()?;
    let content = serde_json::to_string_pretty(prefs)
        .map_err(|e| format!("Failed to serialize UI preferences: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to save UI preferences: {}", e))
}

pub fn load_all_profiles() -> Result<Vec<Profile>, String> {
    let dir = get_profiles_dir()?;
    let mut profiles = Vec::new();

    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read profiles directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read profile {}: {}", name, e))?;
                    match autoeq::parse_autoeq_text(&content) {
                        Ok(data) => {
                            profiles.push(Profile {
                                name: name.to_string(),
                                data,
                            });
                        }
                        Err(e) => {
                            log::warn!("Failed to parse profile {}: {}", name, e);
                        }
                    }
                }
            }
        }
    }

    // Sort profiles by name
    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(profiles)
}

pub fn save_profile(name: &str, data: &PEQData) -> Result<(), String> {
    let dir = get_profiles_dir()?;

    // Sanitize name - simple version for portability
    let sanitized_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect();

    if sanitized_name.is_empty() {
        return Err("Invalid profile name".into());
    }

    let filename = format!("{}.txt", sanitized_name);
    let path = dir.join(filename);

    let content = autoeq::peq_to_autoeq(data);
    fs::write(path, content).map_err(|e| format!("Failed to save profile: {}", e))?;

    Ok(())
}

pub fn delete_profile(name: &str) -> Result<(), String> {
    let dir = get_profiles_dir()?;
    let filename = format!("{}.txt", name);
    let path = dir.join(filename);

    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete profile: {}", e))?;
    }

    Ok(())
}
