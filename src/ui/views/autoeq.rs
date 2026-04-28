use crate::ui::messages::Message;
use crate::ui::state::MainWindow;
use crate::ui::theme::{self, TOKYO_NIGHT_MUTED};
use crate::ui::tokens::{SPACE_12, SPACE_16, SPACE_8, TYPE_TINY};
use crate::ui::views::action_button;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn view_autoeq(state: &MainWindow) -> Element<'_, Message> {
    let is_busy = state.operation_lock.is_pulling || state.operation_lock.is_pushing;

    container(
        column![
            text("AUTO-EQ").size(TYPE_TINY).color(TOKYO_NIGHT_MUTED),
            column![
                row![
                    action_button("Import Clipboard")
                        .on_press_maybe(if is_busy { None } else { Some(Message::ImportFromClipboard) })
                        .style(theme::pill_secondary_button)
                        .width(Length::Fill),
                    action_button("Import File")
                        .on_press_maybe(if is_busy { None } else { Some(Message::ImportFromFilePressed) })
                        .style(theme::pill_secondary_button)
                        .width(Length::Fill),
                ].spacing(SPACE_8),
                row![
                    action_button("Export Clipboard")
                        .on_press_maybe(if is_busy { None } else { Some(Message::ExportAutoEQPressed) })
                        .style(theme::pill_secondary_button)
                        .width(Length::Fill),
                    action_button("Export File")
                        .on_press_maybe(if is_busy { None } else { Some(Message::ExportToFilePressed) })
                        .style(theme::pill_secondary_button)
                        .width(Length::Fill),
                ].spacing(SPACE_8),
            ].spacing(SPACE_8),
        ]
        .spacing(SPACE_12),
    )
    .padding(SPACE_16)
    .style(theme::card_style)
    .width(Length::Fill)
    .into()
}
