
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_RED, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_2, SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_LABEL, TYPE_TITLE, TYPE_TINY};
use crate::ui::views::action_button;
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
                text(format!("BAND {}", i + 1))
                    .size(TYPE_TINY)
                    .color(accent_color)
                    .width(Length::Fill)
                    .center(),
                pick_list(
                    &[
                        crate::models::FilterType::LowShelf,
                        crate::models::FilterType::Peak,
                        crate::models::FilterType::HighShelf
                    ][..],
                    Some(band.filter_type),
                    move |t| Message::BandTypeChanged(i, t),
                )
                .width(Length::Fill)
                .style(theme::m3_input_pick_list)
                .text_size(10),
                
                column![
                    text("FREQ").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    text_input(
                        "",
                        state.editor_state
                            .input_buffer
                            .get_freq(i)
                            .as_deref()
                            .unwrap_or(&format!("{}", band.freq))
                    )
                    .on_input(move |s| Message::BandFreqInput(i, s))
                    .on_submit(Message::BandFreqInputCommit(i))
                    .style(theme::m3_outlined_input)
                    .size(TYPE_LABEL),
                    if let Some(err) = freq_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(TYPE_TINY)
                    }
                ].spacing(SPACE_2),

                column![
                    text("GAIN").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    text_input(
                        "",
                        state.editor_state
                            .input_buffer
                            .get_gain(i)
                            .as_deref()
                            .unwrap_or(&format!("{:.1}", band.gain))
                    )
                    .on_input(move |s| Message::BandGainInput(i, s))
                    .on_submit(Message::BandGainInputCommit(i))
                    .style(theme::m3_outlined_input)
                    .size(TYPE_LABEL),
                     if let Some(err) = gain_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(TYPE_TINY)
                    }
                ].spacing(SPACE_2),

                column![
                    text("Q").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
                    text_input(
                        "",
                        state.editor_state
                            .input_buffer
                            .get_q(i)
                            .as_deref()
                            .unwrap_or(&format!("{:.2}", band.q))
                    )
                    .on_input(move |s| Message::BandQInput(i, s))
                    .on_submit(Message::BandQInputCommit(i))
                    .style(theme::m3_outlined_input)
                    .size(TYPE_LABEL),
                    if let Some(err) = q_error {
                        text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED)
                    } else {
                        text("").size(TYPE_TINY)
                    }
                ].spacing(SPACE_2),
            ]
            .spacing(SPACE_8)
            .padding(SPACE_8);

            container(band_content)
                .width(Length::Fixed(100.0))
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

    container(column![
        if is_busy {
            Element::from(
                container(text("Device sync in progress...").size(TYPE_CAPTION).color(TOKYO_NIGHT_WARNING))
                    .padding(Padding { top: 0.0, right: 0.0, bottom: SPACE_8, left: 0.0 })
            )
        } else {
            text("").into()
        },
        bands_row
    ])
    .padding(SPACE_12)
    .width(Length::Fill)
    .into()
}

pub fn view_advanced_filters_section(state: &MainWindow) -> Element<'_, Message> {
    let expanded = state.editor_state.advanced_filters_expanded;
    let toggle_text = if expanded {
        "Hide advanced filters"
    } else {
        "Show advanced filters"
    };

    let heading = row![
        column![
            text("Advanced filter controls")
                .size(TYPE_TITLE)
                .color(TOKYO_NIGHT_PRIMARY),
            text("Manual PEQ editing for advanced users")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED),
        ]
        .spacing(SPACE_4),
        container(text("")).width(Length::Fill),
        action_button(toggle_text)
            .on_press(Message::ToggleAdvancedFilters(!expanded))
            .style(theme::pill_secondary_button),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(SPACE_12);

    let body: Element<Message> = if expanded {
        view_bands(state)
    } else {
        container(
            text("AutoEQ import/export is recommended for most users.")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_MUTED),
        )
        .padding([SPACE_12, SPACE_8])
        .into()
    };

    container(column![heading, body].spacing(SPACE_12))
        .padding(SPACE_16)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}
