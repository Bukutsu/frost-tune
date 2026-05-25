// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::eq::PEQData;
use crate::error::Result;
use crate::storage::{self, Profile};
use std::path::Path;

/// Facade wrapping the file-based profile and preset persistence engine.
#[derive(Debug, Clone, Copy, Default)]
pub struct PresetService;

impl PresetService {
    pub async fn load_all(&self) -> Result<(Vec<Profile>, Vec<String>)> {
        storage::load_all_profiles().await
    }

    pub async fn save(&self, name: &str, data: &PEQData) -> Result<()> {
        storage::save_profile(name, data).await
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        storage::delete_profile(name).await
    }

    pub async fn import(&self, path: &Path) -> Result<Profile> {
        storage::import_profile(path).await
    }

    pub async fn export(&self, path: &Path, data: &PEQData) -> Result<()> {
        storage::export_profile(path, data).await
    }

    /// Opens the profile directory using the system default file manager.
    pub fn open_directory(&self) -> Result<()> {
        storage::open_profiles_dir()
    }

    /// Returns the current modification time of the profiles directory.
    pub fn get_directory_mtime(&self) -> Option<std::time::SystemTime> {
        storage::get_profiles_dir_mtime()
    }

    /// Returns a human-readable display string of the profiles directory path.
    pub fn get_directory_display(&self) -> String {
        storage::get_profiles_dir_display()
    }
}
