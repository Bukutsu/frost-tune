use iced::Color;

use crate::ui::theme::{
    TOKYO_NIGHT_BG, TOKYO_NIGHT_BG_DARK, TOKYO_NIGHT_BG_HIGHLIGHT, TOKYO_NIGHT_BLUE,
    TOKYO_NIGHT_COMMENT, TOKYO_NIGHT_FG, TOKYO_NIGHT_GREEN, TOKYO_NIGHT_RED, TOKYO_NIGHT_YELLOW,
};

// Material 3 role-oriented tokens mapped to Tokyo Night.
pub const M3_COLOR_SURFACE: Color = TOKYO_NIGHT_BG_HIGHLIGHT;
pub const M3_COLOR_SURFACE_VARIANT: Color = TOKYO_NIGHT_BG_DARK;
pub const M3_COLOR_BACKGROUND: Color = TOKYO_NIGHT_BG;
pub const M3_COLOR_ON_SURFACE: Color = TOKYO_NIGHT_FG;
pub const M3_COLOR_ON_SURFACE_VARIANT: Color = TOKYO_NIGHT_COMMENT;
pub const M3_COLOR_PRIMARY: Color = TOKYO_NIGHT_BLUE;
pub const M3_COLOR_SUCCESS: Color = TOKYO_NIGHT_GREEN;
pub const M3_COLOR_WARNING: Color = TOKYO_NIGHT_YELLOW;
pub const M3_COLOR_ERROR: Color = TOKYO_NIGHT_RED;

// Window-class helpers for adaptive layout decisions.
pub const WINDOW_NARROW_MAX: f32 = 999.0;
pub const WINDOW_MEDIUM_MAX: f32 = 1279.0;

// Spacing scale (8pt baseline with compact variants).
pub const SPACE_4: f32 = 4.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_24: f32 = 24.0;
