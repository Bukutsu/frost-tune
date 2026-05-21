// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

mod diagnostics;
mod paths;
mod profiles;
mod settings;

pub use diagnostics::{append_diagnostics_log, load_recent_diagnostics};
pub use paths::{get_diagnostics_log_path, get_profiles_dir_display, get_profiles_dir_mtime};
pub use profiles::{
    delete_profile, export_profile, import_profile, load_all_profiles, open_profiles_dir,
    save_profile, Profile,
};
pub use settings::{load_settings, save_settings, Settings};
