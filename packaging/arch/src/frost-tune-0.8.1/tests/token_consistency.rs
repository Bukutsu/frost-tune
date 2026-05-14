use frost_tune::ui::theme::{
    self, BAND_ROW_MIN_HEIGHT, BAND_ROW_PADDING, BUTTON_PILL_RADIUS, INPUT_RADIUS, SPACE_8,
    SURFACE_BG, TOKYO_NIGHT_BG_HIGHLIGHT, TYPE_BODY,
};
use iced::widget::button;

fn linear_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn contrast_ratio(a: iced::Color, b: iced::Color) -> f32 {
    let luma = |color: iced::Color| {
        let r = linear_channel(color.r);
        let g = linear_channel(color.g);
        let b = linear_channel(color.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    };

    let l1 = luma(a);
    let l2 = luma(b);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn test_token_consistency() {
    assert_eq!(SPACE_8, 8.0);
    assert_eq!(TYPE_BODY, 16.0);
    assert_eq!(SURFACE_BG, TOKYO_NIGHT_BG_HIGHLIGHT);
}

#[test]
fn test_band_density() {
    assert!(BAND_ROW_MIN_HEIGHT >= 40.0);
    assert_eq!(BAND_ROW_MIN_HEIGHT % 4.0, 0.0);
    assert_eq!(BAND_ROW_PADDING % 4.0, 0.0);
}

#[test]
fn test_shape_semantics_tokens() {
    assert!(BUTTON_PILL_RADIUS >= 999.0);
    assert!((4.0..=8.0).contains(&INPUT_RADIUS));
}

#[test]
fn test_disabled_button_contrast_wcag_aa() {
    let app_theme = theme::theme();
    let disabled = theme::pill_secondary_button(&app_theme, button::Status::Disabled);

    let background = match disabled.background {
        Some(iced::Background::Color(c)) => c,
        _ => panic!("Expected disabled button to have a color background"),
    };

    let ratio = contrast_ratio(disabled.text_color, background);
    assert!(
        ratio >= 3.0,
        "Disabled button contrast ratio is {ratio:.2}, expected >= 3.0"
    );
}
