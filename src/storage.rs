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

fn get_base_dir() -> Result<PathBuf, String> {
    let base_dir = dirs::data_dir()
        .ok_or("Failed to get standard data directory")?
        .join("frost-tune");

    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)
            .map_err(|e| format!("Failed to create application data directory: {}", e))?;
    }

    Ok(base_dir)
}

fn get_profiles_dir() -> Result<PathBuf, String> {
    let profiles_dir = get_base_dir()?.join("profiles");

    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    Ok(profiles_dir)
}

fn get_ui_preferences_path() -> Result<PathBuf, String> {
    Ok(get_base_dir()?.join("ui_preferences.json"))
}

pub fn get_diagnostics_log_path() -> Result<PathBuf, String> {
    Ok(get_base_dir()?.join("diagnostics.log"))
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
    
    let base_dir = get_base_dir()?;
    let tmp_path = base_dir.join(".ui_preferences.json.tmp");
    
    fs::write(&tmp_path, content).map_err(|e| format!("Failed to write temp UI preferences: {}", e))?;
    fs::rename(&tmp_path, &path).map_err(|e| {
        if let Err(cleanup_err) = fs::remove_file(&tmp_path) {
            log::warn!("Failed to clean up temp file after rename error: {}", cleanup_err);
        }
        format!("Failed to finalize UI preferences save: {}", e)
    })?;

    if let Ok(dir_file) = fs::File::open(&base_dir) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}

pub fn get_profiles_dir_mtime() -> Option<std::time::SystemTime> {
    get_profiles_dir().ok().and_then(|dir| {
        fs::metadata(dir).ok().and_then(|m| m.modified().ok())
    })
}

pub fn load_all_profiles() -> Result<(Vec<Profile>, Vec<String>), String> {
    let dir = get_profiles_dir()?;
    let mut profiles = Vec::new();
    let mut errors = Vec::new();

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
                            errors.push(format!("Profile '{}' failed to parse: {}", name, e));
                        }
                    }
                }
            }
        }
    }

    // Sort profiles by name
    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok((profiles, errors))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect()
}

pub fn save_profile(name: &str, data: &PEQData) -> Result<(), String> {
    let dir = get_profiles_dir()?;

    let sanitized_name = sanitize_name(name);

    if sanitized_name.is_empty() {
        return Err("Invalid profile name".into());
    }

    let filename = format!("{}.txt", sanitized_name);
    let path = dir.join(&filename);
    let tmp_path = dir.join(format!(".{}.tmp", sanitized_name));

    let content = autoeq::peq_to_autoeq(data);
    fs::write(&tmp_path, &content).map_err(|e| format!("Failed to write temp profile: {}", e))?;
    fs::rename(&tmp_path, &path).map_err(|e| {
        if let Err(cleanup_err) = fs::remove_file(&tmp_path) {
            log::warn!("Failed to clean up temp file after rename error: {}", cleanup_err);
        }
        format!("Failed to finalize profile save: {}", e)
    })?;

    if let Ok(dir_file) = fs::File::open(&dir) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}

pub fn delete_profile(name: &str) -> Result<(), String> {
    let dir = get_profiles_dir()?;
    let sanitized_name = sanitize_name(name);
    let filename = format!("{}.txt", sanitized_name);
    let path = dir.join(filename);

    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete profile: {}", e))?;
    }

    Ok(())
}

pub fn import_profile(path: &std::path::Path) -> Result<Profile, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read profile file: {}", e))?;
    let data = autoeq::parse_autoeq_text(&content)
        .map_err(|e| format!("Failed to parse profile: {}", e))?;
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Profile")
        .to_string();
    Ok(Profile { name, data })
}

pub fn export_profile(path: &std::path::Path, data: &PEQData) -> Result<(), String> {
    let content = autoeq::peq_to_autoeq(data);
    fs::write(path, content)
        .map_err(|e| format!("Failed to write profile file: {}", e))
}

pub fn open_profiles_dir() -> Result<(), String> {
    let dir = get_profiles_dir()?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}

pub fn append_diagnostics_log(line: &str) -> Result<(), String> {
    let path = get_diagnostics_log_path()?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open diagnostics log: {}", e))?;
    use std::io::Write;
    file.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write diagnostics log: {}", e))?;
    file.write_all(b"\n")
        .map_err(|e| format!("Failed to finalize diagnostics log: {}", e))?;
    Ok(())
}

pub fn load_recent_diagnostics(limit: usize) -> Result<Vec<String>, String> {
    let path = get_diagnostics_log_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read diagnostics log: {}", e))?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.len() > limit {
        lines = lines.split_off(lines.len() - limit);
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hello World"), "Hello World");
        assert_eq!(sanitize_name("Profile_1-2"), "Profile_1-2");
        assert_eq!(sanitize_name("Bad@Name#$"), "BadName");
        assert_eq!(sanitize_name(""), "");
    }
}
