pub mod autoeq;
pub mod bands;
pub mod confirm_dialog;
pub mod diagnostics;
pub mod graph_panel;
pub mod header;
pub mod presets_preamp;
pub mod status_banner;

use crate::ui::messages::Message;
use crate::ui::tokens::{BUTTON_HORIZONTAL_PADDING, BUTTON_VERTICAL_PADDING, TYPE_LABEL};
use iced::widget::{button, text};

pub fn action_button<'a>(label: &'a str) -> iced::widget::Button<'a, Message> {
    button(text(label).size(TYPE_LABEL)).padding([BUTTON_VERTICAL_PADDING, BUTTON_HORIZONTAL_PADDING])
}
