use crate::models::FilterType;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_2, SPACE_32, SPACE_4, SPACE_8, TYPE_LABEL, TYPE_TINY};
use iced::widget::{
    button, checkbox, column, container, responsive, row, slider, text, text_input,
};
use iced::{Background, Color, Element, Length, Padding};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    responsive(move |size| {
        if size.width < 1100.0 {
            // Single column for narrow/medium widths
            let col = render_band_column(0, &state.editor_state.filters, state, is_busy);
            container(col)
                .padding(SPACE_12)
                .width(Length::Fill)
                .align_x(iced::Alignment::Center)
                .style(theme::card_style)
                .into()
        } else {
            // Two columns for wide widths
            let left_col = render_band_column(0, &state.editor_state.filters[0..5], state, is_busy);
            let right_col =
                render_band_column(5, &state.editor_state.filters[5..10], state, is_busy);
            let content = row![left_col, right_col].spacing(SPACE_32).padding(SPACE_8);
            container(content)
                .padding(SPACE_12)
                .width(Length::Fill)
                .align_x(iced::Alignment::Center)
                .style(theme::card_style)
                .into()
        }
    })
    .into()
}

fn render_band_column<'a>(
    start_index: usize,
    filters: &'a [crate::models::filter::Filter],
    state: &'a MainWindow,
    is_busy: bool,
) -> Element<'a, Message> {
    let mut col = column![render_header_row()].spacing(SPACE_4);

    for (i, band) in filters.iter().enumerate() {
        let actual_index = start_index + i;
        col = col.push(render_band_row(actual_index, band, state, is_busy));
    }

    col.into()
}

fn render_header_row<'a>() -> Element<'a, Message> {
    column![
        row![
            text("BAND")
                .size(TYPE_TINY)
                .color(theme::TOKYO_NIGHT_FG)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .width(Length::Fixed(40.0)),
            text("ON")
                .size(TYPE_TINY)
                .color(theme::TOKYO_NIGHT_FG)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .width(Length::Fixed(30.0)),
            container(
                text("TYPE")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            )
            .padding([0.0, 5.0])
            .width(Length::Fixed(160.0)),
            container(
                text("FREQ (Hz)")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            )
            .padding([0.0, 5.0])
            .width(Length::Fixed(85.0)),
            container(
                text("GAIN (dB)")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            )
            .padding([0.0, 5.0])
            .width(Length::Fill),
            container(
                text("Q")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            )
            .padding([0.0, 5.0])
            .width(Length::Fixed(60.0)),
        ]
        .spacing(SPACE_4)
        .padding(Padding {
            top: 0.0,
            right: SPACE_4,
            bottom: SPACE_4,
            left: SPACE_4,
        }),
        container(iced::widget::Space::new().width(Length::Fill).height(1.0))
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(Color {
                    a: 0.2,
                    ..theme::TOKYO_NIGHT_MUTED
                })),
                ..Default::default()
            })
    ]
    .into()
}

fn render_band_row<'a>(
    i: usize,
    band: &'a crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
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

    let gain_range = state.gain_range();

    let type_buttons = row(FilterType::ALL
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
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(20.0))
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
        .collect::<Vec<Element<Message>>>())
    .spacing(SPACE_2);

    let gain_slider = slider(gain_range.0..=gain_range.1, band.gain, move |v| {
        if is_busy {
            Message::None
        } else {
            Message::BandGainChanged(i, v)
        }
    })
    .step(crate::models::constants::GAIN_STEP)
    .width(Length::Fill)
    .style(theme::slider_style);

    let freq_cell = column![
        {
            let input = text_input(
                "",
                state
                    .editor_state
                    .input_buffer
                    .get_freq_input(i)
                    .unwrap_or(&format!("{}", band.freq)),
            )
            .style(theme::m3_filled_input)
            .size(TYPE_LABEL);
            if is_busy {
                input
            } else {
                input
                    .on_input(move |s| Message::BandFreqInput(i, s))
                    .on_submit(Message::BandFreqInputCommit(i))
            }
        },
        if let Some(err) = freq_error {
            text(err).size(TYPE_TINY).color(theme::TOKYO_NIGHT_RED)
        } else {
            text("").size(1)
        }
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(85.0));

    let gain_cell = column![
        gain_slider,
        {
            let input = text_input(
                "",
                state
                    .editor_state
                    .input_buffer
                    .get_gain_input(i)
                    .unwrap_or(&format!("{:.1}", band.gain)),
            )
            .style(theme::m3_filled_input)
            .size(TYPE_LABEL);
            if is_busy {
                input
            } else {
                input
                    .on_input(move |s| Message::BandGainInput(i, s))
                    .on_submit(Message::BandGainInputCommit(i))
            }
        },
        if let Some(err) = gain_error {
            text(err).size(TYPE_TINY).color(theme::TOKYO_NIGHT_RED)
        } else {
            text("").size(1)
        }
    ]
    .spacing(SPACE_2)
    .width(Length::Fill);

    let q_cell = column![
        {
            let input = text_input(
                "",
                state
                    .editor_state
                    .input_buffer
                    .get_q_input(i)
                    .unwrap_or(&format!("{:.2}", band.q)),
            )
            .style(theme::m3_filled_input)
            .size(TYPE_LABEL);
            if is_busy {
                input
            } else {
                input
                    .on_input(move |s| Message::BandQInput(i, s))
                    .on_submit(Message::BandQInputCommit(i))
            }
        },
        if let Some(err) = q_error {
            text(err).size(TYPE_TINY).color(theme::TOKYO_NIGHT_RED)
        } else {
            text("").size(1)
        }
    ]
    .spacing(SPACE_2)
    .width(Length::Fixed(60.0));

    row![
        text(format!("{}", i + 1))
            .size(TYPE_LABEL)
            .color(accent_color)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .width(Length::Fixed(40.0)),
        container(
            checkbox(is_active)
                .on_toggle(move |en| {
                    if is_busy {
                        Message::None
                    } else {
                        Message::BandEnabledToggled(i, en)
                    }
                })
                .size(16)
                .style(theme::checkbox_style)
        )
        .width(Length::Fixed(30.0)),
        container(type_buttons).width(Length::Fixed(160.0)),
        freq_cell,
        gain_cell,
        q_cell,
    ]
    .spacing(SPACE_4)
    .align_y(iced::Alignment::Center)
    .padding(Padding {
        top: SPACE_2,
        right: SPACE_4,
        bottom: SPACE_2,
        left: SPACE_4,
    })
    .into()
}
