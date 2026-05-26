// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::components::editor::ConfirmAction;
use crate::ui::messages::{AutoEqMessage, EditorMessage, Message, ProfilesMessage};
use crate::ui::state::AppState;
use crate::ui::views;
use iced::{widget::container, Element, Length};

pub fn with_modal_overlay<'a>(
    state: &'a AppState,
    main_view: Element<'a, Message>,
) -> Element<'a, Message> {
    if let Some(dialog) = match state.editor.session.pending_confirm {
        ConfirmAction::ResetFilters => Some(views::confirm_dialog::view_confirm_dialog(
            "Reset Filters?".to_string(),
            "This will reset all 10 bands to default values and set global gain to 0.".to_string(),
            "Reset",
            Message::Editor(EditorMessage::ConfirmResetFilters),
            true,
        )),
        ConfirmAction::DeleteProfile => Some(views::confirm_dialog::view_confirm_dialog(
            "Delete Profile?".to_string(),
            "Are you sure you want to delete this profile? This cannot be undone.".to_string(),
            "Delete",
            Message::Profiles(ProfilesMessage::ConfirmDeleteProfile),
            true,
        )),
        ConfirmAction::ImportAutoEQ { ref data, ref default_name } => {
            let count = data.filters.iter().filter(|f| f.enabled).count();
            let message = format!(
                "This will import {} filters and set global gain to {:.1}dB.\n\nCurrent unsaved settings will be replaced.",
                count, data.global_gain,
            );
            Some(views::confirm_dialog::view_import_dialog(
                "Import Profile".to_string(),
                message,
                state.editor.session.import_name_input.as_str(),
                default_name.as_str(),
                &state.editor.ui.profiles,
                state.editor.ui.selected_profile_name.as_deref(),
                state.editor.session.import_temporary,
                "Import",
                Message::AutoEq(AutoEqMessage::ConfirmImportWithName),
            ))
        },
        ConfirmAction::OverwriteProfile { ref name, .. } => Some(views::confirm_dialog::view_confirm_dialog(
            "Overwrite Profile?".to_string(),
            format!("Profile '{}' already exists. Overwrite?", name),
            "Overwrite",
            Message::Profiles(ProfilesMessage::ConfirmOverwriteProfile),
            true,
        )),
        ConfirmAction::PullDevice => Some(views::confirm_dialog::view_confirm_dialog(
            "Pull from Device?".to_string(),
            "You have unsaved changes. Pulling from the device will replace your current editor settings with the hardware configuration. Continue?".to_string(),
            "Discard & Pull",
            Message::Editor(EditorMessage::ConfirmPullPressed),
            false,
        )),
        ConfirmAction::PushToDevice => {
            let active = state.editor.data.filters.iter().filter(|f| f.enabled).count();
            let gain = state.editor.data.global_gain;
            Some(views::confirm_dialog::view_confirm_dialog(
                "Push to Device?".to_string(),
                format!(
                    "This will push {} active bands (global gain: {} dB) to the hardware.\n\nThis cannot be undone.",
                    active, gain
                ),
                "Push",
                Message::Editor(EditorMessage::ConfirmPushPressed),
                false,
            ))
        }
        ConfirmAction::ForceReset => Some(views::confirm_dialog::view_confirm_dialog(
            "Force Reset to Flat?".to_string(),
            "This will bypass normal safeguards and push a flat EQ (all bands disabled) and 0dB gain to the device. Use this only if the hardware state appears corrupted.".to_string(),
            "Force Reset",
            Message::Editor(EditorMessage::ConfirmForceResetPressed),
            true,
        )),
        ConfirmAction::LoadProfile { ref name } => Some(views::confirm_dialog::view_confirm_dialog(
            "Unsaved Changes".to_string(),
            format!("You have unsaved changes. Loading '{}' will replace your current editor settings. Continue?", name),
            "Discard & Load",
            Message::Profiles(ProfilesMessage::ConfirmLoadProfile),
            true,
        )),
        ConfirmAction::ExitWithUnsavedChanges(id) => Some(views::confirm_dialog::view_exit_dialog(
            "Unsaved Changes".to_string(),
            "You have unsaved EQ changes. Save before exiting?".to_string(),
            "Save & Exit",
            Message::SaveAndExit(id),
            "Exit Anyway",
            Message::ConfirmExit(id),
        )),
        ConfirmAction::None => None,
    } {
        iced::widget::stack![
            main_view,
            container(dialog)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(iced::Color { a: 0.8, ..crate::ui::tokens::COLOR_SURFACE_DIM }.into()),
                    ..Default::default()
                })
        ]
        .into()
    } else {
        main_view
    }
}
