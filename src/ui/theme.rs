use iced::theme::Palette;
use iced::widget::{button, container, pick_list, text_input};
use iced::{color, Background, Border, Color, Theme};

pub const SPACE_8: f32 = 8.0;
pub const TYPE_BODY: f32 = 16.0;
pub const SURFACE_BG: Color = TOKYO_NIGHT_BG_HIGHLIGHT;
pub const BAND_ROW_MIN_HEIGHT: f32 = 40.0;
pub const BAND_ROW_PADDING: f32 = 8.0;
pub const BAND_LABEL_WIDTH: f32 = 20.0;
pub const CARD_RADIUS: f32 = 16.0;
pub const HEADER_CARD_RADIUS: f32 = 24.0;
pub const BUTTON_PILL_RADIUS: f32 = 999.0;
pub const INPUT_RADIUS: f32 = 6.0;

// Official Tokyo Night palette (origin: folke/tokyonight.nvim)
pub const TOKYO_NIGHT_BG: Color = color!(0x1a1b26);
pub const TOKYO_NIGHT_BG_DARK: Color = color!(0x16161e);
pub const TOKYO_NIGHT_BG_HIGHLIGHT: Color = color!(0x292e42);
pub const TOKYO_NIGHT_TERMINAL_BLACK: Color = color!(0x414868);
pub const TOKYO_NIGHT_FG: Color = color!(0xc0caf5);
pub const TOKYO_NIGHT_FG_DARK: Color = color!(0xa9b1d6);
pub const TOKYO_NIGHT_COMMENT: Color = color!(0x565f89);
pub const TOKYO_NIGHT_BLUE: Color = color!(0x7aa2f7);
pub const TOKYO_NIGHT_CYAN: Color = color!(0x7dcfff);
pub const TOKYO_NIGHT_GREEN: Color = color!(0x9ece6a);
pub const TOKYO_NIGHT_MAGENTA: Color = color!(0xbb9af7);
pub const TOKYO_NIGHT_ORANGE: Color = color!(0xff9e64);
pub const TOKYO_NIGHT_YELLOW: Color = color!(0xe0af68);
pub const TOKYO_NIGHT_RED: Color = color!(0xf7768e);

// Material role mapping
pub const TOKYO_NIGHT_BACKGROUND: Color = TOKYO_NIGHT_BG;
pub const TOKYO_NIGHT_SURFACE: Color = TOKYO_NIGHT_BG_HIGHLIGHT;
pub const TOKYO_NIGHT_PRIMARY: Color = TOKYO_NIGHT_BLUE;
pub const TOKYO_NIGHT_SECONDARY: Color = TOKYO_NIGHT_MAGENTA;
pub const TOKYO_NIGHT_SUCCESS: Color = TOKYO_NIGHT_GREEN;
pub const TOKYO_NIGHT_ERROR: Color = TOKYO_NIGHT_RED;
pub const TOKYO_NIGHT_TEXT: Color = TOKYO_NIGHT_FG;
pub const TOKYO_NIGHT_MUTED: Color = TOKYO_NIGHT_COMMENT;
pub const TOKYO_NIGHT_WARNING: Color = TOKYO_NIGHT_YELLOW;

pub const GRAPH_BG: Color = TOKYO_NIGHT_BG_DARK;
pub const ACCENT_VIBRANT: Color = TOKYO_NIGHT_CYAN;
pub const SURFACE_DARK: Color = color!(0x1f2335);

pub fn theme() -> Theme {
    Theme::custom(
        "Tokyo Night".to_string(),
        Palette {
            background: TOKYO_NIGHT_BACKGROUND,
            text: TOKYO_NIGHT_TEXT,
            primary: TOKYO_NIGHT_PRIMARY,
            success: TOKYO_NIGHT_SUCCESS,
            warning: TOKYO_NIGHT_WARNING,
            danger: TOKYO_NIGHT_ERROR,
        },
    )
}

pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_DARK)),
        border: Border {
            color: Color {
                a: 0.15,
                ..TOKYO_NIGHT_TERMINAL_BLACK
            },
            width: 1.0,
            radius: CARD_RADIUS.into(),
        },
        ..Default::default()
    }
}

pub fn header_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TOKYO_NIGHT_BG_DARK)),
        border: Border {
            color: Color {
                a: 0.3,
                ..TOKYO_NIGHT_TERMINAL_BLACK
            },
            width: 1.0,
            radius: 0.0.into(), // Header spans top
        },
        ..Default::default()
    }
}

pub fn pill_primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = BUTTON_PILL_RADIUS.into();
    enforce_disabled_button_contrast(style, status)
}

pub fn pill_secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    style.border.radius = BUTTON_PILL_RADIUS.into();
    enforce_disabled_button_contrast(style, status)
}

pub fn pill_danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = BUTTON_PILL_RADIUS.into();
    enforce_disabled_button_contrast(style, status)
}

pub fn pill_text_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.border.radius = BUTTON_PILL_RADIUS.into();
    enforce_disabled_button_contrast(style, status)
}

pub fn pill_outlined_danger_button(_theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(_theme, status);
    let (text_alpha, border_alpha, bg_alpha) = match status {
        button::Status::Disabled => (0.45, 0.3, 0.04),
        button::Status::Hovered => (1.0, 0.9, 0.14),
        button::Status::Pressed => (1.0, 1.0, 0.18),
        _ => (0.92, 0.7, 0.1),
    };
    style.text_color = Color {
        a: text_alpha,
        ..TOKYO_NIGHT_ERROR
    };
    style.background = Some(Background::Color(Color {
        a: bg_alpha,
        ..TOKYO_NIGHT_ERROR
    }));
    style.border = Border {
        color: Color {
            a: border_alpha,
            ..TOKYO_NIGHT_ERROR
        },
        width: 1.0,
        radius: BUTTON_PILL_RADIUS.into(),
    };
    enforce_disabled_button_contrast(style, status)
}

fn enforce_disabled_button_contrast(
    mut style: button::Style,
    status: button::Status,
) -> button::Style {
    if matches!(status, button::Status::Disabled) {
        style.text_color = TOKYO_NIGHT_FG_DARK;
        style.background = Some(Background::Color(Color {
            a: 0.88,
            ..TOKYO_NIGHT_TERMINAL_BLACK
        }));
        style.border = Border {
            color: Color {
                a: 0.72,
                ..TOKYO_NIGHT_FG_DARK
            },
            width: 1.0,
            radius: style.border.radius,
        };
    }

    style
}

pub fn m3_input_pick_list(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let (border_color, border_width) = match status {
        pick_list::Status::Active => (
            Color {
                a: 0.5,
                ..TOKYO_NIGHT_TERMINAL_BLACK
            },
            1.0,
        ),
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => (
            Color {
                a: 0.72,
                ..TOKYO_NIGHT_PRIMARY
            },
            1.2,
        ),
    };

    pick_list::Style {
        text_color: TOKYO_NIGHT_FG,
        placeholder_color: Color {
            a: 0.9,
            ..TOKYO_NIGHT_FG_DARK
        },
        handle_color: TOKYO_NIGHT_FG_DARK,
        background: Background::Color(TOKYO_NIGHT_BG_HIGHLIGHT),
        border: Border {
            color: border_color,
            width: border_width,
            radius: INPUT_RADIUS.into(),
        },
    }
}

pub fn m3_outlined_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let (border_color, border_width) = match status {
        text_input::Status::Active => (
            Color {
                a: 0.5,
                ..TOKYO_NIGHT_TERMINAL_BLACK
            },
            1.0,
        ),
        text_input::Status::Hovered => (
            Color {
                a: 0.72,
                ..TOKYO_NIGHT_PRIMARY
            },
            1.2,
        ),
        text_input::Status::Focused { .. } => (
            Color {
                a: 0.95,
                ..TOKYO_NIGHT_PRIMARY
            },
            1.6,
        ),
        text_input::Status::Disabled => (
            Color {
                a: 0.3,
                ..TOKYO_NIGHT_TERMINAL_BLACK
            },
            1.0,
        ),
    };

    text_input::Style {
        background: Background::Color(TOKYO_NIGHT_BG_HIGHLIGHT),
        border: Border {
            color: border_color,
            width: border_width,
            radius: INPUT_RADIUS.into(),
        },
        icon: TOKYO_NIGHT_FG,
        placeholder: Color {
            a: 0.9,
            ..TOKYO_NIGHT_FG_DARK
        },
        value: TOKYO_NIGHT_FG,
        selection: Color {
            a: 0.55,
            ..TOKYO_NIGHT_PRIMARY
        },
    }
}

pub fn m3_filled_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = m3_outlined_input(_theme, status);
    let underline_color = match status {
        text_input::Status::Focused { .. } => TOKYO_NIGHT_PRIMARY,
        _ => Color {
            a: 0.45,
            ..TOKYO_NIGHT_TERMINAL_BLACK
        },
    };
    style.border = Border {
        color: underline_color,
        width: 1.0,
        radius: INPUT_RADIUS.into(),
    };
    style.background = Background::Color(Color {
        a: 1.0,
        ..TOKYO_NIGHT_BG_HIGHLIGHT
    });
    style
}
