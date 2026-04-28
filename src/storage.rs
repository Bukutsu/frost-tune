use crate::autoeq;
use crate::models::PEQData;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub data: PEQData,
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
    let path = dir.join(filename);

    let content = autoeq::peq_to_autoeq(data);
    fs::write(path, content).map_err(|e| format!("Failed to save profile: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Valid Profile 1"), "Valid Profile 1");
        assert_eq!(sanitize_name("My_Profile-2"), "My_Profile-2");
        assert_eq!(sanitize_name("Bad/Profile\\Name"), "BadProfileName");
        assert_eq!(sanitize_name("Profile (V1)"), "Profile V1");
        assert_eq!(sanitize_name(""), "");
    }
}
