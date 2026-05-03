use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED, TOKYO_NIGHT_PRIMARY, TOKYO_NIGHT_RED, TOKYO_NIGHT_WARNING};
use crate::ui::tokens::{SPACE_12, SPACE_2, SPACE_4, SPACE_8, TYPE_CAPTION, TYPE_LABEL, TYPE_TINY};
use iced::widget::{column, container, pick_list, row, scrollable, text, text_input};
use iced::{Background, Color, Element, Length, Padding};

pub fn view_bands(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    // Header row
    let header_row = row![
        text("BAND").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED).width(Length::Fixed(60.0)),
        text("TYPE").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED).width(Length::Fixed(120.0)),
        text("FREQ (Hz)").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED).width(Length::Fixed(80.0)),
        text("GAIN (dB)").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED).width(Length::Fixed(80.0)),
        text("Q").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED).width(Length::Fixed(80.0)),
    ].spacing(SPACE_8).padding(Padding { top: 0.0, right: SPACE_8, bottom: SPACE_4, left: SPACE_8 });

    let mut band_list: Vec<Element<Message>> = vec![header_row.into()];

    for (i, band) in state.editor_state.filters.iter().enumerate() {
        let freq_error = state.editor_state.input_buffer.get_freq_error(i);
        let gain_error = state.editor_state.input_buffer.get_gain_error(i);
        let q_error = state.editor_state.input_buffer.get_q_error(i);

        let is_active = band.enabled;
        let accent_color = if is_active { TOKYO_NIGHT_PRIMARY } else { TOKYO_NIGHT_MUTED };

        let type_picker = pick_list(
            &[
                crate::models::FilterType::LowShelf,
                crate::models::FilterType::Peak,
                crate::models::FilterType::HighShelf
            ][..],
            Some(band.filter_type),
            move |t| {
                if is_busy { Message::None } else { Message::BandTypeChanged(i, t) }
            },
        )
        .width(Length::Fill)
        .style(theme::m3_input_pick_list)
        .text_size(10);

        let freq_cell = column![
            {
                let input = text_input("", state.editor_state.input_buffer.get_freq(i).as_deref().unwrap_or(&format!("{}", band.freq)))
                    .style(theme::m3_filled_input)
                    .size(TYPE_LABEL);
                if is_busy { input } else { input.on_input(move |s| Message::BandFreqInput(i, s)).on_submit(Message::BandFreqInputCommit(i)) }
            },
            if let Some(err) = freq_error { text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED) } else { text("").size(1) }
        ].spacing(SPACE_2).width(Length::Fixed(80.0));

        let gain_cell = column![
            {
                let input = text_input("", state.editor_state.input_buffer.get_gain(i).as_deref().unwrap_or(&format!("{:.1}", band.gain)))
                    .style(theme::m3_filled_input)
                    .size(TYPE_LABEL);
                if is_busy { input } else { input.on_input(move |s| Message::BandGainInput(i, s)).on_submit(Message::BandGainInputCommit(i)) }
            },
            if let Some(err) = gain_error { text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED) } else { text("").size(1) }
        ].spacing(SPACE_2).width(Length::Fixed(80.0));

        let q_cell = column![
            {
                let input = text_input("", state.editor_state.input_buffer.get_q(i).as_deref().unwrap_or(&format!("{:.2}", band.q)))
                    .style(theme::m3_filled_input)
                    .size(TYPE_LABEL);
                if is_busy { input } else { input.on_input(move |s| Message::BandQInput(i, s)).on_submit(Message::BandQInputCommit(i)) }
            },
            if let Some(err) = q_error { text(err).size(TYPE_TINY).color(TOKYO_NIGHT_RED) } else { text("").size(1) }
        ].spacing(SPACE_2).width(Length::Fixed(80.0));

        let band_row = row![
            text(format!("BAND {}", i + 1)).size(TYPE_LABEL).color(accent_color).width(Length::Fixed(60.0)),
            container(type_picker).width(Length::Fixed(120.0)),
            freq_cell,
            gain_cell,
            q_cell,
        ].spacing(SPACE_8).align_y(iced::Alignment::Center).padding(Padding { top: SPACE_4, right: SPACE_8, bottom: SPACE_4, left: SPACE_8 });

        // Apply tint
        let bg_color = if state.editor_state.is_autoeq_active {
            Color { a: 0.05, ..TOKYO_NIGHT_PRIMARY }
        } else if is_active {
            theme::TOKYO_NIGHT_BG_HIGHLIGHT
        } else {
            Color::TRANSPARENT
        };

        band_list.push(
            container(band_row)
                .width(Length::Fill)
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: iced::Border {
                        color: if is_active { Color { a: 0.3, ..TOKYO_NIGHT_PRIMARY } } else { Color::TRANSPARENT },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        );
    }

    let mut content = column![].spacing(SPACE_8);

    if is_busy {
        content = content.push(
            text("Device sync in progress...").size(TYPE_CAPTION).color(TOKYO_NIGHT_WARNING)
        );
    }

    content = content.push(
        scrollable(column(band_list).spacing(SPACE_4))
    );

    container(content)
        .padding(SPACE_12)
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}