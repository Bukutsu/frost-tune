// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::tokens::{
    COLOR_ERROR, COLOR_ON_PRIMARY, COLOR_ON_SURFACE, COLOR_ON_SURFACE_VARIANT, COLOR_OUTLINE,
    COLOR_OUTLINE_VARIANT, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_SURFACE, COLOR_SURFACE_DIM,
    COLOR_WARNING, ELEVATION_0, ELEVATION_1, ELEVATION_2, SHAPE_EXTRA_SMALL,
    STATE_DISABLED_CONTAINER_OPACITY, STATE_DISABLED_CONTENT_OPACITY, STATE_HOVER_OPACITY,
    STATE_PRESSED_OPACITY,
};
use iced::theme::Palette;
use iced::widget::{button, checkbox, container, pick_list, slider, text_input};
use iced::{color, Background, Border, Color, Theme};

// ── Reference Tokens (Raw Tokyo Night Palette) ─────────────────────────
// Only tokens.rs color mappings and this file's style functions should use these.
// Views import semantic tokens from tokens.rs.
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

// ── Iced Theme ─────────────────────────────────────────────────────────

pub fn theme() -> Theme {
    Theme::custom(
        "Tokyo Night".to_string(),
        Palette {
            background: ELEVATION_0,
            text: COLOR_ON_SURFACE,
            primary: COLOR_PRIMARY,
            success: COLOR_SUCCESS,
            warning: COLOR_WARNING,
            danger: COLOR_ERROR,
        },
    )
}

// ── Container Styles ───────────────────────────────────────────────────

pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ELEVATION_1)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn header_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ELEVATION_0)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn tooltip_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ELEVATION_2)),
        border: Border {
            color: COLOR_OUTLINE,
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: Some(COLOR_ON_SURFACE),
        ..Default::default()
    }
}

// ── Button Styles ──────────────────────────────────────────────────────

pub fn m3_filled_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = SHAPE_EXTRA_SMALL.into();
    match status {
        button::Status::Hovered => {
            style.background = Some(Background::Color(Color {
                a: STATE_HOVER_OPACITY,
                ..COLOR_ON_PRIMARY
            }));
        }
        button::Status::Pressed => {
            style.background = Some(Background::Color(Color {
                a: STATE_PRESSED_OPACITY,
                ..COLOR_ON_PRIMARY
            }));
        }
        _ => {}
    }
    enforce_disabled_button_contrast(style, status)
}

pub fn m3_tonal_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    style.border.radius = SHAPE_EXTRA_SMALL.into();
    match status {
        button::Status::Hovered => {
            style.background = Some(Background::Color(Color {
                a: STATE_HOVER_OPACITY,
                ..COLOR_ON_SURFACE
            }));
        }
        button::Status::Pressed => {
            style.background = Some(Background::Color(Color {
                a: STATE_PRESSED_OPACITY,
                ..COLOR_ON_SURFACE
            }));
        }
        _ => {}
    }
    enforce_disabled_button_contrast(style, status)
}

pub fn m3_filled_button_error(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = SHAPE_EXTRA_SMALL.into();

    match status {
        button::Status::Active => {
            style.background = Some(Background::Color(COLOR_ERROR));
            style.text_color = COLOR_ON_PRIMARY;
        }
        button::Status::Hovered => {
            style.background = Some(Background::Color(Color {
                a: STATE_HOVER_OPACITY,
                ..COLOR_ERROR
            }));
            style.text_color = COLOR_ON_PRIMARY;
        }
        button::Status::Pressed => {
            style.background = Some(Background::Color(Color {
                a: STATE_PRESSED_OPACITY,
                ..COLOR_ERROR
            }));
            style.text_color = COLOR_ON_PRIMARY;
        }
        _ => {}
    }

    enforce_disabled_button_contrast(style, status)
}

pub fn m3_text_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.border.radius = SHAPE_EXTRA_SMALL.into();

    if matches!(status, button::Status::Hovered) {
        style.background = Some(Background::Color(Color {
            a: STATE_HOVER_OPACITY,
            ..COLOR_ON_SURFACE
        }));
        style.border.width = 1.0;
        style.border.color = COLOR_OUTLINE_VARIANT;
    }

    enforce_disabled_button_contrast(style, status)
}

pub fn profile_row_style(
    _theme: &Theme,
    status: button::Status,
    is_selected: bool,
) -> button::Style {
    let mut style = button::text(_theme, status);
    style.border.radius = SHAPE_EXTRA_SMALL.into();
    style.border.width = 0.0;

    if is_selected {
        style.background = Some(Background::Color(COLOR_SURFACE));
        style.text_color = TOKYO_NIGHT_BLUE;
    } else if matches!(status, button::Status::Hovered) {
        style.background = Some(Background::Color(Color {
            a: STATE_HOVER_OPACITY,
            ..COLOR_ON_SURFACE
        }));
    }

    enforce_disabled_button_contrast(style, status)
}

pub fn m3_outlined_button(_theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(_theme, status);
    let (text_alpha, border_alpha, bg_alpha) = match status {
        button::Status::Disabled => (0.45, 0.3, 0.04),
        button::Status::Hovered => (1.0, 0.9, STATE_PRESSED_OPACITY),
        button::Status::Pressed => (1.0, 1.0, 0.15),
        _ => (0.92, 0.7, STATE_HOVER_OPACITY),
    };
    style.text_color = Color {
        a: text_alpha,
        ..COLOR_PRIMARY
    };
    style.background = Some(Background::Color(Color {
        a: bg_alpha,
        ..COLOR_PRIMARY
    }));
    style.border = Border {
        color: Color {
            a: border_alpha,
            ..COLOR_PRIMARY
        },
        width: 1.0,
        radius: SHAPE_EXTRA_SMALL.into(),
    };
    enforce_disabled_button_contrast(style, status)
}

pub fn m3_outlined_button_error(_theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(_theme, status);
    let (text_alpha, border_alpha, bg_alpha) = match status {
        button::Status::Disabled => (0.45, 0.3, 0.04),
        button::Status::Hovered => (1.0, 0.9, 0.14),
        button::Status::Pressed => (1.0, 1.0, 0.18),
        _ => (0.92, 0.7, STATE_PRESSED_OPACITY),
    };
    style.text_color = Color {
        a: text_alpha,
        ..COLOR_ERROR
    };
    style.background = Some(Background::Color(Color {
        a: bg_alpha,
        ..COLOR_ERROR
    }));
    style.border = Border {
        color: Color {
            a: border_alpha,
            ..COLOR_ERROR
        },
        width: 1.0,
        radius: SHAPE_EXTRA_SMALL.into(),
    };
    enforce_disabled_button_contrast(style, status)
}

fn enforce_disabled_button_contrast(
    mut style: button::Style,
    status: button::Status,
) -> button::Style {
    if matches!(status, button::Status::Disabled) {
        style.text_color = COLOR_ON_SURFACE_VARIANT;
        style.background = Some(Background::Color(Color {
            a: 0.88,
            ..COLOR_OUTLINE
        }));
        style.border = Border {
            color: Color {
                a: 0.72,
                ..COLOR_ON_SURFACE_VARIANT
            },
            width: 1.0,
            radius: style.border.radius,
        };
    }

    style
}

// ── Input Styles ───────────────────────────────────────────────────────

pub fn m3_input_pick_list(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let (border_color, border_width) = match status {
        pick_list::Status::Active => (
            Color {
                a: 0.5,
                ..COLOR_OUTLINE
            },
            1.0,
        ),
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => (COLOR_PRIMARY, 2.0),
    };

    pick_list::Style {
        text_color: COLOR_ON_SURFACE,
        placeholder_color: Color {
            a: 0.9,
            ..COLOR_ON_SURFACE_VARIANT
        },
        handle_color: COLOR_ON_SURFACE_VARIANT,
        background: Background::Color(COLOR_SURFACE),
        border: Border {
            color: border_color,
            width: border_width,
            radius: SHAPE_EXTRA_SMALL.into(),
        },
    }
}

pub fn m3_outlined_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let (border_color, border_width) = match status {
        text_input::Status::Active => (
            Color {
                a: 0.5,
                ..COLOR_OUTLINE
            },
            1.0,
        ),
        text_input::Status::Hovered => (
            Color {
                a: 0.72,
                ..COLOR_PRIMARY
            },
            1.2,
        ),
        text_input::Status::Focused { .. } => (COLOR_PRIMARY, 2.0),
        text_input::Status::Disabled => (
            Color {
                a: 0.3,
                ..COLOR_OUTLINE
            },
            1.0,
        ),
    };

    text_input::Style {
        background: Background::Color(COLOR_SURFACE),
        border: Border {
            color: border_color,
            width: border_width,
            radius: SHAPE_EXTRA_SMALL.into(),
        },
        icon: COLOR_ON_SURFACE,
        placeholder: Color {
            a: 0.9,
            ..COLOR_ON_SURFACE_VARIANT
        },
        value: COLOR_ON_SURFACE,
        selection: Color {
            a: 0.55,
            ..COLOR_PRIMARY
        },
    }
}

pub fn m3_filled_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = m3_outlined_input(_theme, status);
    let underline_color = match status {
        text_input::Status::Focused { .. } => COLOR_PRIMARY,
        _ => Color {
            a: 0.45,
            ..COLOR_OUTLINE
        },
    };
    style.border = Border {
        color: underline_color,
        width: if matches!(status, text_input::Status::Focused { .. }) {
            2.0
        } else {
            1.0
        },
        radius: SHAPE_EXTRA_SMALL.into(),
    };
    style.background = Background::Color(if matches!(status, text_input::Status::Focused { .. }) {
        COLOR_SURFACE_DIM
    } else {
        COLOR_SURFACE
    });
    style
}

pub fn m3_transparent_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = m3_outlined_input(_theme, status);

    let (border_color, border_width, bg_color) = match status {
        text_input::Status::Focused { .. } => (COLOR_PRIMARY, 2.0, COLOR_SURFACE_DIM),
        text_input::Status::Hovered => (
            Color {
                a: 0.3,
                ..COLOR_OUTLINE
            },
            1.0,
            COLOR_SURFACE,
        ),
        _ => (Color::TRANSPARENT, 0.0, Color::TRANSPARENT),
    };

    style.border = Border {
        color: border_color,
        width: border_width,
        radius: SHAPE_EXTRA_SMALL.into(),
    };
    style.background = Background::Color(bg_color);
    style
}

// ── Checkbox Style ─────────────────────────────────────────────────────

pub fn checkbox_style(_theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let is_checked = match status {
        checkbox::Status::Active { is_checked } => is_checked,
        checkbox::Status::Hovered { is_checked } => is_checked,
        checkbox::Status::Disabled { is_checked } => is_checked,
    };

    let is_disabled = matches!(status, checkbox::Status::Disabled { .. });

    let icon_color = if is_disabled {
        Color {
            a: STATE_DISABLED_CONTENT_OPACITY,
            ..COLOR_ON_PRIMARY
        }
    } else {
        COLOR_ON_PRIMARY
    };

    let border_color = if is_checked {
        if is_disabled {
            Color {
                a: STATE_DISABLED_CONTENT_OPACITY,
                ..COLOR_PRIMARY
            }
        } else {
            COLOR_PRIMARY
        }
    } else if is_disabled {
        Color {
            a: STATE_DISABLED_CONTAINER_OPACITY,
            ..COLOR_OUTLINE
        }
    } else {
        COLOR_OUTLINE
    };

    checkbox::Style {
        background: if is_checked {
            if is_disabled {
                Background::Color(Color {
                    a: STATE_DISABLED_CONTENT_OPACITY,
                    ..COLOR_PRIMARY
                })
            } else {
                Background::Color(COLOR_PRIMARY)
            }
        } else if is_disabled {
            Background::Color(Color {
                a: STATE_DISABLED_CONTAINER_OPACITY,
                ..COLOR_SURFACE
            })
        } else {
            Background::Color(Color::TRANSPARENT)
        },
        icon_color,
        border: Border {
            radius: SHAPE_EXTRA_SMALL.into(),
            width: 1.0,
            color: border_color,
        },
        text_color: if is_disabled {
            Some(Color {
                a: STATE_DISABLED_CONTENT_OPACITY,
                ..COLOR_ON_SURFACE
            })
        } else {
            Some(COLOR_ON_SURFACE)
        },
    }
}

// ── Slider Styles ──────────────────────────────────────────────────────

pub fn slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(TOKYO_NIGHT_BLUE),
                Background::Color(COLOR_SURFACE_DIM),
            ),
            width: 4.0,
            border: Border {
                radius: 2.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(TOKYO_NIGHT_BLUE),
            border_width: 1.0,
            border_color: TOKYO_NIGHT_BLUE,
        },
    }
}

fn gain_color(gain: f64) -> Color {
    if gain > 0.05 {
        TOKYO_NIGHT_ORANGE
    } else if gain < -0.05 {
        TOKYO_NIGHT_CYAN
    } else {
        TOKYO_NIGHT_COMMENT
    }
}

pub fn gain_slider_style(
    gain: f64,
    is_active: bool,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme: &Theme, _status: slider::Status| {
        let mut accent = gain_color(gain);
        if !is_active {
            accent.a *= 0.3;
        }
        slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(accent),
                    Background::Color(COLOR_SURFACE_DIM),
                ),
                width: 4.0,
                border: Border {
                    radius: 2.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: Background::Color(accent),
                border_width: 1.0,
                border_color: accent,
            },
        }
    }
}
