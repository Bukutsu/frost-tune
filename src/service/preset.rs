// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::eq::PEQData;
use crate::error::Result;
use crate::storage::{self, Profile};
use std::path::Path;

/// Facade wrapping the file-based profile and preset persistence engine.
#[derive(Debug, Clone, Copy)]
pub struct PresetService;

impl PresetService {
    /// Loads all saved AutoEQ/EqualizerAPO profiles from the storage directory.
    pub fn load_all() -> Result<(Vec<Profile>, Vec<String>)> {
        storage::load_all_profiles()
    }

    /// Saves a profile under a specified name with the given PEQ parameters.
    pub fn save(name: &str, data: &PEQData) -> Result<()> {
        storage::save_profile(name, data)
    }

    /// Deletes a profile by name.
    pub fn delete(name: &str) -> Result<()> {
        storage::delete_profile(name)
    }

    /// Imports a profile from a specified filesystem path.
    pub fn import(path: &Path) -> Result<Profile> {
        storage::import_profile(path)
    }

    /// Exports a profile to a specified filesystem path.
    pub fn export(path: &Path, data: &PEQData) -> Result<()> {
        storage::export_profile(path, data)
    }

    /// Opens the profile directory using the system default file manager.
    pub fn open_directory() -> Result<()> {
        storage::open_profiles_dir()
    }

    /// Returns the current modification time of the profiles directory.
    pub fn get_directory_mtime() -> Option<std::time::SystemTime> {
        storage::get_profiles_dir_mtime()
    }

    /// Returns a human-readable display string of the profiles directory path.
    pub fn get_directory_display() -> String {
        storage::get_profiles_dir_display()
    }
}
