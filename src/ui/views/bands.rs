use crate::models::{MAX_BAND_GAIN, MIN_BAND_GAIN};
use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_ERROR, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_2, SPACE_4, SPACE_8, TYPE_BODY, TYPE_CAPTION, TYPE_LABEL, TYPE_TITLE};
use crate::ui::views::action_button;
use iced::widget::{column, container, pick_list, row, scrollable, slider, text, text_input};
use iced::{Element, Length};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    let busy_notice: Element<Message> = if is_busy {
        container(
            text("Device sync in progress... controls temporarily locked")
                .size(TYPE_LABEL)
                .color(TOKYO_NIGHT_WARNING),
        )
        .padding(SPACE_12)
        .into()
    } else {
        text("").into()
    };

    let band_list: Vec<Element<Message>> = state
        .editor_state
        .filters
        .iter()
        .enumerate()
        .map(|(i, band)| {
            let freq_error = state.editor_state.input_buffer.get_freq_error(i);
            let gain_error = state.editor_state.input_buffer.get_gain_error(i);
            let q_error = state.editor_state.input_buffer.get_q_error(i);

            let freq_error_display = if let Some(err) = freq_error {
                text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
            } else {
                text("")
            };
            let gain_error_display = if let Some(err) = gain_error {
                text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
            } else {
                text("")
            };
            let q_error_display = if let Some(err) = q_error {
                text(err).size(TYPE_CAPTION).color(TOKYO_NIGHT_ERROR)
            } else {
                text("")
            };

            column![
                row![
                    text(format!("{}", i + 1))
                        .size(TYPE_BODY)
                        .width(Length::Fixed(20.0)),
                    pick_list(
                        &[
                            crate::models::FilterType::LowShelf,
                            crate::models::FilterType::Peak,
                            crate::models::FilterType::HighShelf
                        ][..],
                        Some(band.filter_type),
                        move |t| Message::BandTypeChanged(i, t),
                    )
                    .width(Length::Fixed(110.0))
                    .style(theme::m3_input_pick_list)
                    .text_size(12),
                    row![text_input(
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
                    .width(Length::Fixed(80.0))
                    .size(TYPE_LABEL),]
                    .spacing(SPACE_4)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(2)),
                    row![
                        slider(MIN_BAND_GAIN..=MAX_BAND_GAIN, band.gain, move |v| {
                            Message::BandGainChanged(i, v)
                        })
                        .step(0.1)
                        .width(Length::Fill),
                        text_input(
                            "",
                            state.editor_state
                                .input_buffer
                                .get_gain(i)
                                .as_deref()
                                .unwrap_or(&format!("{:.2}", band.gain))
                        )
                        .on_input(move |s| Message::BandGainInput(i, s))
                        .on_submit(Message::BandGainInputCommit(i))
                        .style(theme::m3_outlined_input)
                        .width(Length::Fixed(60.0))
                        .size(TYPE_LABEL),
                    ]
                    .spacing(SPACE_4)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(4)),
                    row![text_input(
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
                    .width(Length::Fixed(60.0))
                    .size(TYPE_LABEL),]
                    .spacing(SPACE_4)
                    .align_y(iced::Alignment::Center)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(SPACE_4)
                .align_y(iced::Alignment::Center),
                row![
                    text("").width(Length::Fixed(20.0)),
                    text("").width(Length::Fixed(110.0)),
                    freq_error_display.width(Length::FillPortion(2)),
                    gain_error_display.width(Length::FillPortion(4)),
                    q_error_display.width(Length::FillPortion(1)),
                ]
                .spacing(SPACE_4),
            ]
            .spacing(SPACE_2)
            .into()
        })
        .collect();

    let header = row![
        text("#")
            .size(TYPE_LABEL)
            .color(TOKYO_NIGHT_MUTED)
            .width(Length::Fixed(20.0)),
        text("Type")
            .size(TYPE_LABEL)
            .color(TOKYO_NIGHT_MUTED)
            .width(Length::Fixed(110.0)),
        text("Frequency (Hz)")
            .size(TYPE_LABEL)
            .color(TOKYO_NIGHT_MUTED)
            .width(Length::FillPortion(2)),
        row![
            container(text("Gain (dB)").size(TYPE_LABEL).color(TOKYO_NIGHT_MUTED))
                .width(Length::Fill)
                .center_x(Length::Fill),
            container(text("")).width(Length::Fixed(60.0)),
        ]
        .width(Length::FillPortion(4)),
        text("Q")
            .size(TYPE_LABEL)
            .color(TOKYO_NIGHT_MUTED)
            .width(Length::FillPortion(1)),
    ]
    .spacing(SPACE_4)
    .align_y(iced::Alignment::Center);

    container(
        container(column![
            busy_notice,
            header,
            scrollable(column(band_list).spacing(SPACE_8))
        ])
        .max_width(1080),
    )
    .padding([SPACE_12, SPACE_8])
    .style(theme::card_style)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
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
