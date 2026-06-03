// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::FilterType;
use crate::ui::messages::EqSource;
use crate::ui::messages::*;
use crate::ui::state::AppState;
use crate::ui::theme;
use crate::ui::tokens::{
    BANDS_TWO_COLUMN_BREAK, BAND_CHECKBOX_WIDTH, BAND_ENABLE_ICON_WIDTH, BAND_FILTER_BUTTON_HEIGHT,
    BAND_FILTER_BUTTON_WIDTH, BAND_FREQ_INPUT_WIDTH, BAND_GAIN_INPUT_WIDTH, BAND_GAIN_LABEL_WIDTH,
    BAND_Q_INPUT_WIDTH, BAND_TYPE_PICKER_WIDTH, CHECKBOX_SIZE, COLOR_ERROR, COLOR_ON_PRIMARY,
    COLOR_ON_SURFACE, COLOR_ON_SURFACE_VARIANT, COLOR_PRIMARY, SPACE_0, SPACE_1, SPACE_12,
    SPACE_16, SPACE_2, SPACE_24, SPACE_4, SPACE_8, STATE_DISABLED_CONTENT_OPACITY, TYPE_LABEL,
    TYPE_SUBTITLE, TYPE_TINY,
};
use iced::widget::{
    button, checkbox, column, container, responsive, row, slider, text, text_input, tooltip,
};
use iced::{Background, Color, Element, Length, Padding};

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
                Some(Message::AutoEq(AutoEqMessage::ImportFromClipboard))
            })
            .style(theme::m3_filled_button);

    let file_btn = super::icon_action_button(crate::ui::tokens::ICON_IMPORT_FILE, "Open File…")
        .on_press_maybe(if is_busy {
            None
        } else {
            Some(Message::Profiles(ProfilesMessage::ImportFromFilePressed))
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

pub fn view_bands(state: &AppState) -> Element<'_, Message> {
    let is_busy =
        state.connection.operation_lock.is_pulling || state.connection.operation_lock.is_pushing;
    let show_enable = state.supports_per_band_enable();

    let is_empty = state.editor.ui.eq_source == EqSource::Default
        && state.editor.data.peq.filters.iter().all(|f| !f.enabled);

    if is_empty {
        return render_empty_state(is_busy);
    }

    responsive(move |size| {
        if size.width < BANDS_TWO_COLUMN_BREAK {
            // Single column for narrow/medium widths
            let col =
                render_band_column(0, &state.editor.data.peq.filters, state, is_busy, show_enable);
            container(col)
                .style(theme::card_style)
                .padding(SPACE_8)
                .width(Length::Fill)
                .into()
        } else {
            // Two columns for wide widths
            let mid = state.editor.data.peq.filters.len() / 2;
            let col1 = render_band_column(
                0,
                &state.editor.data.peq.filters[..mid],
                state,
                is_busy,
                show_enable,
            );
            let col2 = render_band_column(
                mid,
                &state.editor.data.peq.filters[mid..],
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
    .height(Length::Shrink)
    .into()
}

fn render_band_column<'a>(
    start_index: usize,
    bands: &'a [crate::core::Filter],
    state: &'a AppState,
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

fn filter_type_short_label(ft: FilterType) -> &'static str {
    match ft {
        FilterType::Peak => "PK",
        FilterType::LowShelf => "LS",
        FilterType::HighShelf => "HS",
        FilterType::HighPass => "HP",
        FilterType::LowPass => "LP",
    }
}

fn render_type_buttons<'a>(
    i: usize,
    band: &crate::core::Filter,
    is_busy: bool,
    is_active: bool,
    supported: crate::core::FilterTypeFlags,
) -> Element<'a, Message> {
    row(FilterType::ALL
        .iter()
        .filter(|&&ft| supported.supports(ft))
        .map(|&ft| {
            let is_selected = band.filter_type == ft;
            let label = filter_type_short_label(ft);
            let btn = button(
                container(
                    text(label)
                        .size(TYPE_TINY)
                        .color(if is_selected {
                            if is_active {
                                COLOR_ON_PRIMARY
                            } else {
                                Color {
                                    a: STATE_DISABLED_CONTENT_OPACITY,
                                    ..COLOR_ON_PRIMARY
                                }
                            }
                        } else if is_active {
                            COLOR_ON_SURFACE
                        } else {
                            Color {
                                a: STATE_DISABLED_CONTENT_OPACITY,
                                ..COLOR_ON_SURFACE
                            }
                        })
                        .align_x(iced::Alignment::Center),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(BAND_FILTER_BUTTON_WIDTH))
            .height(Length::Fixed(BAND_FILTER_BUTTON_HEIGHT))
            .padding(SPACE_0)
            .style(move |theme, status| {
                let mut style = theme::m3_text_button(theme, status);
                style.border.width = 0.0;

                if is_selected {
                    style.background = Some(COLOR_PRIMARY.into());
                    style.text_color = COLOR_ON_PRIMARY;
                } else {
                    let base_bg = match status {
                        iced::widget::button::Status::Hovered => crate::ui::tokens::COLOR_OUTLINE,
                        _ => crate::ui::tokens::COLOR_SURFACE,
                    };
                    style.background = Some(base_bg.into());
                }

                if !is_active {
                    if let Some(Background::Color(c)) = &mut style.background {
                        c.a *= STATE_DISABLED_CONTENT_OPACITY;
                    }
                    style.text_color.a *= STATE_DISABLED_CONTENT_OPACITY;
                }
                style
            });

            if is_busy {
                btn.into()
            } else {
                btn.on_press(Message::Editor(EditorMessage::BandTypeChanged(i, ft)))
                    .into()
            }
        })
        .collect::<Vec<Element<Message>>>())
    .spacing(crate::ui::tokens::SPACE_1)
    .into()
}

fn render_freq_cell<'a>(
    i: usize,
    band: &crate::core::Filter,
    state: &'a AppState,
    is_busy: bool,
    freq_error: Option<&'a str>,
    is_active: bool,
) -> Element<'a, Message> {
    column![render_input_field(
        state
            .editor
            .session
            .input_buffer
            .get_freq_input(i)
            .map_or_else(|| format!("{}", band.freq), |s| s.to_string()),
        is_busy,
        freq_error,
        is_active,
        move |s| Message::Editor(EditorMessage::BandFreqInput(i, s)),
        Message::Editor(EditorMessage::BandFreqInputCommit(i)),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(BAND_GAIN_LABEL_WIDTH))
    .into()
}

fn render_gain_cell<'a>(
    i: usize,
    band: &crate::core::Filter,
    state: &'a AppState,
    is_busy: bool,
    gain_error: Option<&'a str>,
    is_active: bool,
) -> Element<'a, Message> {
    let gain_range = state.gain_range();
    let slider = slider(gain_range.0..=gain_range.1, band.gain, move |v| {
        if is_busy {
            Message::NoOp
        } else {
            Message::Editor(EditorMessage::BandGainChanged(i, v))
        }
    })
    .step(crate::core::GAIN_STEP)
    .width(Length::Fill)
    .style(theme::gain_slider_style(band.gain, is_active))
    .on_release(if is_busy {
        Message::NoOp
    } else {
        Message::Editor(EditorMessage::BandGainReleased(i))
    });

    let input = render_input_field_raw(
        state
            .editor
            .session
            .input_buffer
            .get_gain_input(i)
            .map_or_else(|| format!("{:.2}", band.gain), |s| s.to_string()),
        is_busy,
        is_active,
        move |s| Message::Editor(EditorMessage::BandGainInput(i, s)),
        Message::Editor(EditorMessage::BandGainInputCommit(i)),
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
    band: &crate::core::Filter,
    state: &'a AppState,
    is_busy: bool,
    q_error: Option<&'a str>,
    is_active: bool,
) -> Element<'a, Message> {
    column![render_input_field(
        state
            .editor
            .session
            .input_buffer
            .get_q_input(i)
            .map_or_else(|| format!("{:.2}", band.q), |s| s.to_string()),
        is_busy,
        q_error,
        is_active,
        move |s| Message::Editor(EditorMessage::BandQInput(i, s)),
        Message::Editor(EditorMessage::BandQInputCommit(i)),
    )]
    .spacing(SPACE_2)
    .width(Length::Fixed(BAND_Q_INPUT_WIDTH))
    .into()
}

fn render_band_row<'a>(
    i: usize,
    band: &'a crate::core::Filter,
    state: &'a AppState,
    is_busy: bool,
    show_enable: bool,
) -> Element<'a, Message> {
    let freq_error = state.editor.session.input_buffer.get_freq_error(i);
    let gain_error = state.editor.session.input_buffer.get_gain_error(i);
    let q_error = state.editor.session.input_buffer.get_q_error(i);

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
                            Message::NoOp
                        } else {
                            Message::Editor(EditorMessage::BandEnabledToggled(i, en))
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
        container(render_type_buttons(
            i,
            band,
            is_busy,
            is_active,
            state.supported_filter_types(),
        ))
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
