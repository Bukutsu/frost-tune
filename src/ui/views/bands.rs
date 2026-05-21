// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::models::FilterType;
use crate::ui::messages::Message;
use crate::ui::state::{EqSource, MainWindow};
use crate::ui::theme;
use crate::ui::tokens::{
    BANDS_TWO_COLUMN_BREAK, BAND_CHECKBOX_WIDTH, BAND_ENABLE_ICON_WIDTH, BAND_FREQ_INPUT_WIDTH,
    BAND_GAIN_INPUT_WIDTH, BAND_GAIN_LABEL_WIDTH, BAND_Q_INPUT_WIDTH, BAND_TYPE_PICKER_WIDTH,
    CHECKBOX_SIZE, COLOR_ERROR, COLOR_ON_SURFACE, COLOR_ON_SURFACE_VARIANT, COLOR_PRIMARY, SPACE_0,
    SPACE_1, SPACE_12, SPACE_16, SPACE_2, SPACE_24, SPACE_4, SPACE_8,
    STATE_DISABLED_CONTENT_OPACITY, TYPE_LABEL, TYPE_SUBTITLE, TYPE_TINY,
};
use iced::widget::{
    checkbox, column, container, pick_list, responsive, row, slider, text, text_input, tooltip,
};
use iced::{Color, Element, Length, Padding};

fn render_empty_state<'a>(is_busy: bool) -> Element<'a, Message> {
    let title = text("No EQ loaded")
        .size(TYPE_SUBTITLE)
        .color(COLOR_ON_SURFACE)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        });

    let hint = text("Paste an EQ from squig.link, peqdb.com, or any AutoEQ source.")
        .size(TYPE_LABEL)
        .color(COLOR_ON_SURFACE_VARIANT);

    let paste_btn =
        super::icon_action_button(crate::ui::tokens::ICON_IMPORT_CLIPBOARD, "Paste (Ctrl+V)")
            .on_press_maybe(if is_busy {
                None
            } else {
                Some(Message::ImportFromClipboard)
            })
            .style(theme::m3_filled_button);

    let file_btn = super::icon_action_button(crate::ui::tokens::ICON_IMPORT_FILE, "Open File…")
        .on_press_maybe(if is_busy {
            None
        } else {
            Some(Message::ImportFromFilePressed)
        })
        .style(theme::m3_tonal_button);

    let preset_hint = text("Or pick a saved preset on the right →")
        .size(TYPE_TINY)
        .color(COLOR_ON_SURFACE_VARIANT);

    let body = column![
        title,
        hint,
        row![paste_btn, file_btn].spacing(SPACE_8),
        preset_hint,
    ]
    .spacing(SPACE_16)
    .align_x(iced::Alignment::Center);

    container(body)
        .style(theme::card_style)
        .padding(SPACE_24)
        .center_x(Length::Fill)
        .into()
}

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;
    let show_enable = state.supports_per_band_enable();

    let is_empty = state.editor_state.ui.eq_source == EqSource::Default
        && state.editor_state.data.filters.iter().all(|f| !f.enabled);

    if is_empty {
        return render_empty_state(is_busy);
    }

    responsive(move |size| {
        if size.width < BANDS_TWO_COLUMN_BREAK {
            // Single column for narrow/medium widths
            let col = render_band_column(
                0,
                &state.editor_state.data.filters,
                state,
                is_busy,
                show_enable,
            );
            container(col)
                .style(theme::card_style)
                .padding(SPACE_8)
                .width(Length::Fill)
                .into()
        } else {
            // Two columns for wide widths
            let mid = state.editor_state.data.filters.len() / 2;
            let col1 = render_band_column(
                0,
                &state.editor_state.data.filters[..mid],
                state,
                is_busy,
                show_enable,
            );
            let col2 = render_band_column(
                mid,
                &state.editor_state.data.filters[mid..],
                state,
                is_busy,
                show_enable,
            );
            row![
                container(col1)
                    .style(theme::card_style)
                    .padding(SPACE_8)
                    .width(Length::Fill),
                container(col2)
                    .style(theme::card_style)
                    .padding(SPACE_8)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_12)
            .into()
        }
    })
    .into()
}

fn render_band_column<'a>(
    start_index: usize,
    bands: &'a [crate::models::Filter],
    state: &'a MainWindow,
    is_busy: bool,
    show_enable: bool,
) -> Element<'a, Message> {
    let mut col = column![render_header_row(show_enable)].spacing(SPACE_2);

    for (offset, band) in bands.iter().enumerate() {
        let actual_index = start_index + offset;
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
        .color(COLOR_ON_SURFACE)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .width(Length::Fixed(BAND_CHECKBOX_WIDTH))
        .into()];

    if show_enable {
        elements.push(
            text("ON")
                .size(TYPE_TINY)
                .color(COLOR_ON_SURFACE)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .width(Length::Fixed(BAND_ENABLE_ICON_WIDTH))
                .into(),
        );
    }

    elements.push(
        container(
            text("TYPE")
                .size(TYPE_TINY)
                .color(COLOR_ON_SURFACE)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
        )
        .padding([SPACE_0, SPACE_4])
        .width(Length::Fixed(BAND_TYPE_PICKER_WIDTH))
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("FREQ (Hz)")
                    .size(TYPE_TINY)
                    .color(COLOR_ON_SURFACE)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Center frequency of the filter band"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([SPACE_0, SPACE_4])
        .width(Length::Fixed(BAND_FREQ_INPUT_WIDTH))
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("GAIN (dB)")
                    .size(TYPE_TINY)
                    .color(COLOR_ON_SURFACE)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Boost or cut level. Range: +/-10 dB"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([SPACE_0, SPACE_4])
        .width(Length::Fill)
        .into(),
    );
    elements.push(
        container(
            tooltip(
                text("Q")
                    .size(TYPE_TINY)
                    .color(COLOR_ON_SURFACE)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                text("Quality factor. Lower = wider, higher = narrower"),
                tooltip::Position::Bottom,
            )
            .style(theme::tooltip_style),
        )
        .padding([SPACE_0, SPACE_4])
        .width(Length::Fixed(BAND_Q_INPUT_WIDTH))
        .into(),
    );

    column![
        row(elements).spacing(SPACE_4).padding(Padding {
            top: SPACE_0,
            right: SPACE_4,
            bottom: SPACE_4,
            left: SPACE_4,
        }),
        container(iced::widget::rule::horizontal(SPACE_1).style(theme::divider_rule_style))
            .width(Length::Fill)
    ]
    .into()
}

fn render_input_field_raw<'a>(
    value: String,
    is_busy: bool,
    is_active: bool,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    let input = text_input("", &value)
        .font(iced::Font::MONOSPACE)
        .style(move |theme, status| {
            let mut style = theme::m3_transparent_input(theme, status);
            if !is_active {
                style.value.a = STATE_DISABLED_CONTENT_OPACITY;
            }
            style
        })
        .size(TYPE_LABEL);
    let input = if is_busy {
        input
    } else {
        input.on_input(on_input).on_submit(on_submit)
    };
    input.into()
}

fn render_input_field<'a>(
    value: String,
    is_busy: bool,
    error: Option<&'a str>,
    is_active: bool,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    let input = render_input_field_raw(value, is_busy, is_active, on_input, on_submit);
    let error_row: Element<'_, Message> = if let Some(err) = error {
        text(err).size(TYPE_TINY).color(COLOR_ERROR).into()
    } else {
        iced::widget::Space::new().height(TYPE_TINY).into()
    };
    column![input, error_row].spacing(SPACE_2).into()
}

fn render_type_picker<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    is_busy: bool,
    is_active: bool,
) -> Element<'a, Message> {
    if is_busy {
        let dim_color = if is_active {
            COLOR_ON_SURFACE_VARIANT
        } else {
            Color {
                a: STATE_DISABLED_CONTENT_OPACITY,
                ..COLOR_ON_SURFACE_VARIANT
            }
        };
        return container(
            text(band.filter_type.to_string())
                .size(TYPE_LABEL)
                .color(dim_color),
        )
        .padding([SPACE_4, SPACE_8])
        .width(Length::Fill)
        .into();
    }

    pick_list(FilterType::ALL, Some(band.filter_type), move |ft| {
        Message::BandTypeChanged(i, ft)
    })
    .style(theme::m3_input_pick_list)
    .text_size(TYPE_LABEL)
    .padding([SPACE_4, SPACE_8])
    .width(Length::Fill)
    .into()
}

fn render_freq_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    freq_error: Option<&'a str>,
    is_active: bool,
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
        is_active,
        move |s| Message::BandFreqInput(i, s),
        Message::BandFreqInputCommit(i),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(BAND_GAIN_LABEL_WIDTH))
    .into()
}

fn render_gain_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    gain_error: Option<&'a str>,
    is_active: bool,
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
    .style(theme::gain_slider_style(band.gain, is_active))
    .on_release(if is_busy {
        Message::None
    } else {
        Message::BandGainReleased(i)
    });

    let input = render_input_field_raw(
        state
            .editor_state
            .session
            .input_buffer
            .get_gain_input(i)
            .map_or_else(|| format!("{:.2}", band.gain), |s| s.to_string()),
        is_busy,
        is_active,
        move |s| Message::BandGainInput(i, s),
        Message::BandGainInputCommit(i),
    );

    let error_row: Element<'_, Message> = if let Some(err) = gain_error {
        text(err).size(TYPE_TINY).color(COLOR_ERROR).into()
    } else {
        iced::widget::Space::new().height(TYPE_TINY).into()
    };

    let slider_and_input = row![
        slider,
        container(input).width(Length::Fixed(BAND_GAIN_INPUT_WIDTH)),
    ]
    .spacing(SPACE_4)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);

    column![
        slider_and_input,
        row![
            iced::widget::Space::new().width(Length::Fill),
            container(error_row).width(Length::Fixed(BAND_GAIN_INPUT_WIDTH)),
        ]
    ]
    .spacing(SPACE_2)
    .width(Length::Fill)
    .into()
}

fn render_q_cell<'a>(
    i: usize,
    band: &crate::models::filter::Filter,
    state: &'a MainWindow,
    is_busy: bool,
    q_error: Option<&'a str>,
    is_active: bool,
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
        is_active,
        move |s| Message::BandQInput(i, s),
        Message::BandQInputCommit(i),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(BAND_Q_INPUT_WIDTH))
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
        COLOR_PRIMARY
    } else {
        COLOR_ON_SURFACE_VARIANT
    };

    let mut elements: Vec<Element<'a, Message>> = vec![text(format!("{}", i + 1))
        .size(TYPE_LABEL)
        .color(accent_color)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .width(Length::Fixed(BAND_CHECKBOX_WIDTH))
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
                    .size(CHECKBOX_SIZE)
                    .style(theme::checkbox_style),
            )
            .width(Length::Fixed(BAND_ENABLE_ICON_WIDTH))
            .into(),
        );
    }

    elements.push(
        container(render_type_picker(i, band, is_busy, is_active))
            .width(Length::Fixed(BAND_TYPE_PICKER_WIDTH))
            .into(),
    );
    elements.push(render_freq_cell(
        i, band, state, is_busy, freq_error, is_active,
    ));
    elements.push(render_gain_cell(
        i, band, state, is_busy, gain_error, is_active,
    ));
    elements.push(render_q_cell(i, band, state, is_busy, q_error, is_active));

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
