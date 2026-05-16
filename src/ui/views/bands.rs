use crate::models::FilterType;
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_2, SPACE_32, SPACE_4, SPACE_8, TYPE_LABEL, TYPE_TINY};
use iced::widget::{
    button, checkbox, column, container, responsive, row, slider, text, text_input, tooltip,
};
use iced::{Background, Color, Element, Length, Padding};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;
    let show_enable = state.supports_per_band_enable();

    responsive(move |size| {
        if size.width < 1100.0 {
            // Single column for narrow/medium widths
            let col = render_band_column(
                0,
                &state.editor_state.data.filters,
                state,
                is_busy,
                show_enable,
            );
            container(col)
                .padding(SPACE_12)
                .width(Length::Fill)
                .align_x(iced::Alignment::Center)
                .style(theme::card_style)
                .into()
        } else {
            // Two columns for wide widths
            let left_filters = state.editor_state.data.filters.get(0..5).unwrap_or(&[]);
            let right_filters = state.editor_state.data.filters.get(5..10).unwrap_or(&[]);
            let left_col = render_band_column(0, left_filters, state, is_busy, show_enable);
            let right_col = render_band_column(5, right_filters, state, is_busy, show_enable);
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
    show_enable: bool,
) -> Element<'a, Message> {
    let mut col = column![render_header_row(show_enable)].spacing(SPACE_4);

    for (i, band) in filters.iter().enumerate() {
        let actual_index = start_index + i;
        col = col.push(render_band_row(
            actual_index,
            band,
            state,
            is_busy,
            show_enable,
        ));
    }

    col.into()
}

fn render_header_row<'a>(show_enable: bool) -> Element<'a, Message> {
    let mut elements: Vec<Element<'a, Message>> = vec![text("BAND")
        .size(TYPE_TINY)
        .color(theme::TOKYO_NIGHT_FG)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .width(Length::Fixed(40.0))
        .into()];

    if show_enable {
        elements.push(
            text("ON")
                .size(TYPE_TINY)
                .color(theme::TOKYO_NIGHT_FG)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .width(Length::Fixed(30.0))
                .into(),
        );
    }

    elements.push(
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
        .width(Length::Fixed(160.0))
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("FREQ (Hz)")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Center frequency of the filter band"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([0.0, 5.0])
        .width(Length::Fixed(85.0))
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("GAIN (dB)")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Boost or cut level. Range: +/-10 dB"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([0.0, 5.0])
        .width(Length::Fill)
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("Q")
                    .size(TYPE_TINY)
                    .color(theme::TOKYO_NIGHT_FG)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Quality factor. Lower = wider, higher = narrower"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([0.0, 5.0])
        .width(Length::Fixed(60.0))
        .into(),
    );

    column![
        row(elements).spacing(SPACE_4).padding(Padding {
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

fn render_input_field<'a>(
    value: String,
    is_busy: bool,
    error: Option<&'a str>,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    let input = text_input("", &value)
        .style(theme::m3_filled_input)
        .size(TYPE_LABEL);
    let input = if is_busy {
        input
    } else {
        input.on_input(on_input).on_submit(on_submit)
    };
    column![
        input,
        if let Some(err) = error {
            text(err).size(TYPE_TINY).color(theme::TOKYO_NIGHT_RED)
        } else {
            text("").size(1)
        }
    ]
    .spacing(SPACE_2)
    .into()
}

fn render_type_buttons<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    is_busy: bool,
) -> Element<'a, Message> {
    row(FilterType::ALL
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
            .height(Length::Fixed(26.0))
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
    .spacing(SPACE_2)
    .into()
}

fn render_freq_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    freq_error: Option<&'a str>,
) -> Element<'a, Message> {
    column![render_input_field(
        state
            .editor_state
            .session
            .input_buffer
            .get_freq_input(i)
            .map_or_else(|| format!("{}", band.freq), |s| s.to_string()),
        is_busy,
        freq_error,
        move |s| Message::BandFreqInput(i, s),
        Message::BandFreqInputCommit(i),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(85.0))
    .into()
}

fn render_gain_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    gain_error: Option<&'a str>,
) -> Element<'a, Message> {
    let gain_range = state.gain_range();
    let slider = slider(gain_range.0..=gain_range.1, band.gain, move |v| {
        if is_busy {
            Message::None
        } else {
            Message::BandGainChanged(i, v)
        }
    })
    .step(crate::models::constants::GAIN_STEP)
    .width(Length::Fill)
    .style(theme::slider_style);

    row![
        slider,
        container(render_input_field(
            state
                .editor_state
                .session
                .input_buffer
                .get_gain_input(i)
                .map_or_else(|| format!("{:.2}", band.gain), |s| s.to_string()),
            is_busy,
            gain_error,
            move |s| Message::BandGainInput(i, s),
            Message::BandGainInputCommit(i),
        ))
        .width(Length::Fixed(55.0)),
    ]
    .spacing(SPACE_4)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn render_q_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    q_error: Option<&'a str>,
) -> Element<'a, Message> {
    column![render_input_field(
        state
            .editor_state
            .session
            .input_buffer
            .get_q_input(i)
            .map_or_else(|| format!("{:.2}", band.q), |s| s.to_string()),
        is_busy,
        q_error,
        move |s| Message::BandQInput(i, s),
        Message::BandQInputCommit(i),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(60.0))
    .into()
}

fn render_band_row<'a>(
    i: usize,
    band: &'a crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    show_enable: bool,
) -> Element<'a, Message> {
    let freq_error = state.editor_state.session.input_buffer.get_freq_error(i);
    let gain_error = state.editor_state.session.input_buffer.get_gain_error(i);
    let q_error = state.editor_state.session.input_buffer.get_q_error(i);

    let is_active = band.enabled;
    let accent_color = if is_active {
        theme::TOKYO_NIGHT_PRIMARY
    } else {
        theme::TOKYO_NIGHT_MUTED
    };

    let mut elements: Vec<Element<'a, Message>> = vec![text(format!("{}", i + 1))
        .size(TYPE_LABEL)
        .color(accent_color)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .width(Length::Fixed(40.0))
        .into()];

    if show_enable {
        elements.push(
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
                    .style(theme::checkbox_style),
            )
            .width(Length::Fixed(30.0))
            .into(),
        );
    }

    elements.push(
        container(render_type_buttons(i, band, is_busy))
            .width(Length::Fixed(160.0))
            .into(),
    );
    elements.push(render_freq_cell(i, band, state, is_busy, freq_error));
    elements.push(render_gain_cell(i, band, state, is_busy, gain_error));
    elements.push(render_q_cell(i, band, state, is_busy, q_error));

    row(elements)
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
