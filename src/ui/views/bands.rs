use crate::models::FilterType;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_2, SPACE_4, TYPE_LABEL, TYPE_TINY};
use iced::widget::{button, checkbox, column, container, row, text, text_input, vertical_slider};
use iced::{Color, Element, Length, Padding};

const SLIDER_HEIGHT: f32 = 160.0;
const TYPE_BTN_HEIGHT: f32 = 24.0;

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;
    let gain_range = state.gain_range();

    let columns: Vec<Element<Message>> = state
        .editor_state
        .filters
        .iter()
        .enumerate()
        .map(|(i, band)| render_band_column(i, band, state, is_busy, gain_range))
        .collect();

    container(
        row(columns)
            .spacing(SPACE_2)
            .padding(SPACE_4)
            .width(Length::Fill)
            .height(Length::Shrink),
    )
    .padding(SPACE_4)
    .style(theme::card_style)
    .width(Length::Fill)
    .into()
}

fn render_band_column<'a>(
    i: usize,
    band: &'a crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    gain_range: (f64, f64),
) -> Element<'a, Message> {
    let freq_error = state.editor_state.input_buffer.get_freq_error(i);
    let gain_error = state.editor_state.input_buffer.get_gain_error(i);
    let q_error = state.editor_state.input_buffer.get_q_error(i);

    let is_active = band.enabled;
    let accent_color = if is_active {
        theme::TOKYO_NIGHT_PRIMARY
    } else {
        theme::TOKYO_NIGHT_MUTED
    };

    // --- Band number + on/off toggle ---
    let header = row![
        container(
            checkbox(is_active)
                .on_toggle(move |en| {
                    if is_busy {
                        Message::None
                    } else {
                        Message::BandEnabledToggled(i, en)
                    }
                })
                .size(14)
                .style(theme::checkbox_style)
        )
        .center_y(Length::Fill),
        text(format!("{}", i + 1))
            .size(TYPE_LABEL)
            .color(accent_color)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .align_x(iced::Alignment::Center),
    ]
    .spacing(SPACE_2)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);

    // --- Filter type button stack ---
    let type_stack = column(
        FilterType::ALL
            .iter()
            .map(|&ft| {
                let is_selected = band.filter_type == ft;
                let label = ft.short_label();
                let btn = button(
                    container(
                        text(label)
                            .size(TYPE_TINY)
                            .color(if is_selected {
                                theme::TOKYO_NIGHT_BG_DARK
                            } else {
                                theme::TOKYO_NIGHT_FG
                            })
                            .align_x(iced::Alignment::Center),
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
                )
                .height(Length::Fixed(TYPE_BTN_HEIGHT))
                .width(Length::Fill)
                .padding(0)
                .style(move |_theme, status| {
                    let base = if is_selected {
                        iced::widget::button::Style {
                            background: Some(theme::TOKYO_NIGHT_PRIMARY.into()),
                            border: iced::Border {
                                color: theme::TOKYO_NIGHT_PRIMARY,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            text_color: theme::TOKYO_NIGHT_BG_DARK,
                            ..Default::default()
                        }
                    } else {
                        iced::widget::button::Style {
                            background: Some(theme::TOKYO_NIGHT_BG_DARK.into()),
                            border: iced::Border {
                                color: Color {
                                    a: 0.3,
                                    ..theme::TOKYO_NIGHT_MUTED
                                },
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            text_color: theme::TOKYO_NIGHT_FG,
                            ..Default::default()
                        }
                    };
                    match status {
                        iced::widget::button::Status::Hovered if !is_selected => {
                            iced::widget::button::Style {
                                background: Some(theme::TOKYO_NIGHT_BG_HIGHLIGHT.into()),
                                ..base
                            }
                        }
                        iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(
                                Color {
                                    a: 0.8,
                                    ..theme::TOKYO_NIGHT_PRIMARY
                                }
                                .into(),
                            ),
                            ..base
                        },
                        _ => base,
                    }
                });

                if is_busy {
                    btn.into()
                } else {
                    btn.on_press(Message::BandTypeChanged(i, ft)).into()
                }
            })
            .collect::<Vec<Element<Message>>>(),
    )
    .spacing(SPACE_2)
    .width(Length::Fill);

    // --- Vertical gain slider ---
    let gain_slider = vertical_slider(gain_range.0..=gain_range.1, band.gain, move |v| {
        if is_busy {
            Message::None
        } else {
            Message::BandGainChanged(i, v)
        }
    })
    .step(crate::models::constants::GAIN_STEP)
    .height(Length::Fixed(SLIDER_HEIGHT))
    .width(32.0)
    .style(theme::slider_style);

    // --- Gain value display ---
    let gain_display = container(
        text(format!("{:.1} dB", band.gain))
            .size(TYPE_TINY)
            .color(accent_color)
            .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .center_x(Length::Fill);

    // --- Text inputs ---
    let gain_value = state
        .editor_state
        .input_buffer
        .get_gain_input(i)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{:.1}", band.gain));
    let gain_input = compact_input(
        gain_value,
        is_busy,
        move |s| Message::BandGainInput(i, s),
        Message::BandGainInputCommit(i),
        gain_error,
    );

    let freq_value = state
        .editor_state
        .input_buffer
        .get_freq_input(i)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}", band.freq));
    let freq_input = compact_input(
        freq_value,
        is_busy,
        move |s| Message::BandFreqInput(i, s),
        Message::BandFreqInputCommit(i),
        freq_error,
    );

    let q_value = state
        .editor_state
        .input_buffer
        .get_q_input(i)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{:.2}", band.q));
    let q_input = compact_input(
        q_value,
        is_busy,
        move |s| Message::BandQInput(i, s),
        Message::BandQInputCommit(i),
        q_error,
    );

    column![
        header,
        type_stack,
        container(gain_slider)
            .width(Length::Fill)
            .center_x(Length::Fill),
        gain_display,
        gain_input,
        freq_input,
        q_input,
    ]
    .spacing(SPACE_4)
    .align_x(iced::Alignment::Center)
    .width(Length::FillPortion(1))
    .into()
}

fn compact_input<'a>(
    value: String,
    is_busy: bool,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
    error: Option<&'a str>,
) -> Element<'a, Message> {
    let input = text_input("", value.as_str())
        .style(theme::m3_filled_input)
        .size(TYPE_TINY)
        .padding(Padding::new(SPACE_4));

    let input = if is_busy {
        input
    } else {
        input.on_input(on_input).on_submit(on_submit)
    };

    column![
        input.width(Length::Fill),
        if let Some(err) = error {
            text(err).size(TYPE_TINY).color(theme::TOKYO_NIGHT_RED)
        } else {
            text("").size(1)
        }
    ]
    .spacing(SPACE_2)
    .width(Length::Fill)
    .into()
}
