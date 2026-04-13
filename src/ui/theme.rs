use iced::theme::Palette;
use iced::widget::container;
use iced::{color, Background, Border, Color, Theme};

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
        background: Some(Background::Color(TOKYO_NIGHT_SURFACE)),
        border: Border {
            color: TOKYO_NIGHT_TERMINAL_BLACK,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
