use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_RED, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_12, SPACE_2, SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_LABEL, TYPE_TINY};
use iced::widget::{column, container, pick_list, row, scrollable, text, text_input};
use iced::{Background, Element, Length, Padding};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    let band_list: Vec<Element<Message>> = state
        .editor_state
        .filters
        .iter()
        .enumerate()
        .map(|(i, band)| {
            let freq_error = state.editor_state.input_buffer.get_freq_error(i);
            let gain_error = state.editor_state.input_buffer.get_gain_error(i);
            let q_error = state.editor_state.input_buffer.get_q_error(i);

            let is_active = band.enabled;
            let accent_color = if is_active { TOKYO_NIGHT_PRIMARY } else { TOKYO_NIGHT_MUTED };

            let band_content = column![
                // Band title
                text(format!("BAND {}", i + 1))
                    .size(TYPE_TINY)
                    .color(accent_color)
                    .width(Length::Fill)
                    .center(),
                // Filter type picker
                pick_list(
                    &[
                        crate::models::FilterType::LowShelf,
                        crate::models::FilterType::Peak,
                        crate::models::FilterType::HighShelf
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
                .text_size(10),
                // Freq field
                column![
                    text("FREQ").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    {
                        let input = text_input(
                            "",
                            state.editor_state
                                .input_buffer
                                .get_freq(i)
                                .as_deref()
                                .unwrap_or(&format!("{}", band.freq))
                        )
                        .style(theme::m3_outlined_input)
                        .size(TYPE_LABEL);
                        
                        if is_busy {
                            input
                        } else {
                            input.on_input(move |s| Message::BandFreqInput(i, s))
                                 .on_submit(Message::BandFreqInputCommit(i))
                        }
                    },
                    if let Some(err) = freq_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(1)
                    }
                ].spacing(SPACE_2),
                // Gain field
                column![
                    text("GAIN").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    {
                        let input = text_input(
                            "",
                            state.editor_state
                                .input_buffer
                                .get_gain(i)
                                .as_deref()
                                .unwrap_or(&format!("{:.1}", band.gain))
                        )
                        .style(theme::m3_outlined_input)
                        .size(TYPE_LABEL);
                        
                        if is_busy {
                            input
                        } else {
                            input.on_input(move |s| Message::BandGainInput(i, s))
                                 .on_submit(Message::BandGainInputCommit(i))
                        }
                    },
                     if let Some(err) = gain_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(1)
                    }
                ].spacing(SPACE_2),
                // Q field
                column![
                    text("Q").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    {
                        let input = text_input(
                            "",
                            state.editor_state
                                .input_buffer
                                .get_q(i)
                                .as_deref()
                                .unwrap_or(&format!("{:.2}", band.q))
                        )
                        .style(theme::m3_outlined_input)
                        .size(TYPE_LABEL);
                        
                        if is_busy {
                            input
                        } else {
                            input.on_input(move |s| Message::BandQInput(i, s))
                                 .on_submit(Message::BandQInputCommit(i))
                        }
                    },
                    if let Some(err) = q_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(1)
                    }
                ].spacing(SPACE_2),
            ]
            .spacing(SPACE_4)
            .padding(Padding { top: SPACE_8, right: SPACE_8, bottom: SPACE_4, left: SPACE_8 });

            container(band_content)
                .width(Length::Fixed(128.0))
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(if is_active { theme::TOKYO_NIGHT_BG_HIGHLIGHT } else { theme::TOKYO_NIGHT_BG_DARK })),
                    border: iced::Border {
                        color: if is_active { TOKYO_NIGHT_PRIMARY } else { theme::TOKYO_NIGHT_TERMINAL_BLACK },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        })
        .collect();

    let bands_row = scrollable(
        row(band_list).spacing(SPACE_8)
    )
    .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()));

    let mut content = column![].spacing(SPACE_8);

    if is_busy {
        content = content.push(
            text("Device sync in progress...").size(TYPE_CAPTION).color(TOKYO_NIGHT_WARNING)
        );
    }

    content = content.push(bands_row);

    container(content)
        .padding(SPACE_12)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
