use iced::Color;

use crate::ui::theme::{
    TOKYO_NIGHT_BG, TOKYO_NIGHT_BG_DARK, TOKYO_NIGHT_BG_HIGHLIGHT, TOKYO_NIGHT_BLUE,
    TOKYO_NIGHT_CYAN, TOKYO_NIGHT_FG, TOKYO_NIGHT_FG_DARK, TOKYO_NIGHT_GREEN, TOKYO_NIGHT_MAGENTA,
    TOKYO_NIGHT_RED, TOKYO_NIGHT_TERMINAL_BLACK, TOKYO_NIGHT_YELLOW,
};

// ── M3 Color Roles (System Tokens) ─────────────────────────────────────
// Views should reference these semantic tokens, not raw TOKYO_NIGHT_* palette values.
// Raw palette stays in theme.rs for use only in this mapping and theme style functions.

pub const COLOR_PRIMARY: Color = TOKYO_NIGHT_CYAN;
pub const COLOR_ON_PRIMARY: Color = TOKYO_NIGHT_BG_DARK;
pub const COLOR_SECONDARY: Color = TOKYO_NIGHT_MAGENTA;
pub const COLOR_SURFACE: Color = TOKYO_NIGHT_BG_HIGHLIGHT;
pub const COLOR_SURFACE_DIM: Color = TOKYO_NIGHT_BG_DARK;
pub const COLOR_SURFACE_CONTAINER: Color = Color {
    r: 0.122,
    g: 0.137,
    b: 0.208,
    a: 1.0,
}; // #1f2335
pub const COLOR_SURFACE_CONTAINER_HIGH: Color = TOKYO_NIGHT_BG_HIGHLIGHT;
pub const COLOR_ON_SURFACE: Color = TOKYO_NIGHT_FG;
pub const COLOR_ON_SURFACE_VARIANT: Color = TOKYO_NIGHT_FG_DARK;
pub const COLOR_OUTLINE: Color = TOKYO_NIGHT_TERMINAL_BLACK;
pub const COLOR_OUTLINE_VARIANT: Color = Color {
    r: 0.255,
    g: 0.282,
    b: 0.408,
    a: 0.15,
}; // TERMINAL_BLACK at 15%
pub const COLOR_ERROR: Color = TOKYO_NIGHT_RED;
pub const COLOR_SUCCESS: Color = TOKYO_NIGHT_GREEN;
pub const COLOR_WARNING: Color = TOKYO_NIGHT_YELLOW;
pub const COLOR_INFO: Color = TOKYO_NIGHT_BLUE;

// ── M3 Elevation (Tonal Surface Colors) ────────────────────────────────
// Iced has no shadow API — we use tonal surface layering per M3 dark theme spec.
pub const ELEVATION_0: Color = TOKYO_NIGHT_BG; // Level 0: App background
pub const ELEVATION_1: Color = COLOR_SURFACE_CONTAINER; // Level 1: Cards, panels
pub const ELEVATION_2: Color = TOKYO_NIGHT_BG_HIGHLIGHT; // Level 2: Dialogs, menus, tooltips
pub const ELEVATION_3: Color = TOKYO_NIGHT_TERMINAL_BLACK; // Level 3: Interactive hover overlays

// ── M3 Shape Scale ─────────────────────────────────────────────────────
pub const SHAPE_NONE: f32 = 0.0;
pub const SHAPE_EXTRA_SMALL: f32 = 0.0;
pub const SHAPE_SMALL: f32 = 0.0;
pub const SHAPE_MEDIUM: f32 = 12.0;
pub const SHAPE_LARGE: f32 = 16.0;
pub const SHAPE_EXTRA_LARGE: f32 = 28.0;
pub const SHAPE_FULL: f32 = 999.0;

// ── M3 State Layer Opacities ───────────────────────────────────────────
pub const STATE_HOVER_OPACITY: f32 = 0.08;
pub const STATE_PRESSED_OPACITY: f32 = 0.10;
pub const STATE_DISABLED_CONTENT_OPACITY: f32 = 0.38;
pub const STATE_DISABLED_CONTAINER_OPACITY: f32 = 0.12;

// ── Window Classes (Adaptive Layout) ───────────────────────────────────
pub const WINDOW_NARROW_MAX: f32 = 999.0;
pub const WINDOW_MEDIUM_MAX: f32 = 1279.0;
pub const WINDOW_MAX_CONTENT_WIDTH: f32 = 1280.0;
pub const SIDEBAR_WIDTH: f32 = 320.0;

// ── Spacing Scale (4dp base) ───────────────────────────────────────────
pub const SPACE_1: f32 = 1.0;
pub const SPACE_2: f32 = 2.0;
pub const SPACE_4: f32 = 4.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_10: f32 = 10.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_20: f32 = 20.0;
pub const SPACE_24: f32 = 24.0;
pub const SPACE_32: f32 = 32.0;
pub const SPACE_40: f32 = 40.0;

// ── Typography Scale ───────────────────────────────────────────────────
pub const TYPE_DISPLAY: f32 = 32.0;
pub const TYPE_TITLE: f32 = 22.0;
pub const TYPE_SUBTITLE: f32 = 18.0;
pub const TYPE_BODY: f32 = 16.0;
pub const TYPE_LABEL: f32 = 13.0;
pub const TYPE_CAPTION: f32 = 11.0;
pub const TYPE_TINY: f32 = 9.0;

// ── Component Dimensions ───────────────────────────────────────────────
pub const BUTTON_HEIGHT_LARGE: f32 = 48.0;
pub const BUTTON_HEIGHT_MEDIUM: f32 = 40.0;
pub const BUTTON_HEIGHT_SMALL: f32 = 36.0;
pub const BUTTON_HEIGHT_COMPACT: f32 = 32.0;
pub const ICON_BUTTON_SIZE: f32 = 36.0;
pub const BUTTON_VERTICAL_PADDING: f32 = 10.0;
pub const BUTTON_HORIZONTAL_PADDING: f32 = 16.0;
pub const INPUT_HEIGHT: f32 = 36.0;
pub const DIALOG_WIDTH: f32 = 400.0;
pub const DIALOG_WIDTH_SMALL: f32 = 360.0;

// Band table metrics
pub const BAND_ROW_MIN_HEIGHT: f32 = 40.0;
pub const BAND_ROW_PADDING: f32 = 8.0;
pub const BAND_LABEL_WIDTH: f32 = 20.0;
pub const BAND_CHECKBOX_WIDTH: f32 = 40.0;
pub const BAND_ENABLE_ICON_WIDTH: f32 = 30.0;
pub const BAND_TYPE_PICKER_WIDTH: f32 = 160.0;
pub const BAND_FREQ_INPUT_WIDTH: f32 = 85.0;
pub const BAND_Q_INPUT_WIDTH: f32 = 60.0;
pub const BAND_GAIN_INPUT_WIDTH: f32 = 55.0;
pub const BAND_GAIN_LABEL_WIDTH: f32 = 85.0;
pub const BAND_Q_LABEL_WIDTH: f32 = 60.0;
pub const BAND_FILTER_BUTTON_WIDTH: f32 = 28.0;
pub const BAND_FILTER_BUTTON_HEIGHT: f32 = 26.0;
pub const PREAMP_LABEL_WIDTH: f32 = 120.0;
pub const PROFILE_LIST_HEIGHT: f32 = 200.0;
pub const DIAGNOSTICS_LEVEL_WIDTH: f32 = 70.0;
pub const DIAGNOSTICS_TIME_WIDTH: f32 = 40.0;

// ── Icons ──────────────────────────────────────────────────────────────
pub const ICON_FONT: iced::Font = iced::Font::with_name("Material Icons");
pub const ICON_FOLDER: &str = "\u{e2c7}";
pub const ICON_RELOAD: &str = "\u{e5d5}";
pub const ICON_CLOSE: &str = "\u{e5cd}";
pub const ICON_IMPORT_FILE: &str = "\u{e2c4}";
pub const ICON_EXPORT_FILE: &str = "\u{e2c6}";
pub const ICON_IMPORT_CLIPBOARD: &str = "\u{e14f}";
pub const ICON_EXPORT_CLIPBOARD: &str = "\u{e173}";
pub const ICON_CHECK_CIRCLE: &str = "\u{e86c}";
pub const ICON_INFO: &str = "\u{e88e}";
pub const ICON_WARNING: &str = "\u{e002}";
pub const ICON_ERROR: &str = "\u{e000}";
pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../../assets/MaterialIcons-Regular.ttf");
