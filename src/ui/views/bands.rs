use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme;
use crate::ui::tokens::{SPACE_12, SPACE_2, SPACE_32, SPACE_4, SPACE_8, TYPE_LABEL, TYPE_TINY};
use iced::widget::{checkbox, column, container, pick_list, row, text, text_input};
use iced::{Background, Color, Element, Length, Padding};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // Split bands into two columns (1-5 and 6-10)
    let left_bands = &state.editor_state.filters[0..5];
    let right_bands = &state.editor_state.filters[5..10];

    let left_col = render_band_column(0, left_bands, state, is_busy);
    let right_col = render_band_column(5, right_bands, state, is_busy);

    let content = row![left_col, right_col].spacing(SPACE_32).padding(SPACE_8);

    container(content)
        .padding(SPACE_12)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center)
        .style(theme::card_style)
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
            .width(Length::Fixed(110.0)),
            container(
                text("FREQ")
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
                text("GAIN")
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
        .spacing(SPACE_8)
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

    let type_picker = pick_list(
        &[
            crate::models::FilterType::LowShelf,
            crate::models::FilterType::Peak,
            crate::models::FilterType::HighShelf,
        ][..],
        Some(band.filter_type),
        move |t| {
            if is_busy {
                Message::None
            } else {
                Message::BandTypeChanged(i, t)
            }
        },
    )
    .width(Length::Fill)
    .style(theme::m3_input_pick_list)
    .text_size(10);

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
    .width(Length::Fixed(85.0));

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

    let band_row = row![
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
        container(type_picker).width(Length::Fixed(110.0)),
        freq_cell,
        gain_cell,
        q_cell,
    ]
    .spacing(SPACE_8)
    .align_y(iced::Alignment::Center)
    .padding(Padding {
        top: SPACE_2,
        right: SPACE_4,
        bottom: SPACE_2,
        left: SPACE_4,
    });

    band_row.into()
}
