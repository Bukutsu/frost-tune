// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::ui::messages::{AutoEqMessage, EditorMessage, Message, ProfilesMessage};
use crate::ui::state::AppState;
use iced::{keyboard, Subscription};
use std::time::Duration;

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    use iced::time;

    async fn tick() -> Message {
        Message::Tick(std::time::Instant::now())
    }

    let tick_sub = time::repeat(|| Box::pin(tick()), Duration::from_secs(2));
    let close_sub = iced::window::close_requests().map(Message::CloseRequested);
    let keyboard_sub = keyboard::listen().filter_map(|event| {
        if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
            if modifiers.control() {
                if modifiers.shift() && key == keyboard::Key::Character("Z".into()) {
                    return Some(Message::Editor(EditorMessage::Redo));
                }
                if key == keyboard::Key::Character("z".into()) {
                    return Some(Message::Editor(EditorMessage::Undo));
                }
                if key == keyboard::Key::Character("s".into()) {
                    return Some(Message::Profiles(ProfilesMessage::SaveProfilePressed));
                }
                if key == keyboard::Key::Character("r".into()) {
                    return Some(Message::Editor(EditorMessage::PullPressed));
                }
                if modifiers.shift() && key == keyboard::Key::Character("r".into()) {
                    return Some(Message::Editor(EditorMessage::ResetFiltersPressed));
                }
                if key == keyboard::Key::Named(keyboard::key::Named::Enter) {
                    return Some(Message::Editor(EditorMessage::PushPressed));
                }
                if key == keyboard::Key::Character("v".into()) {
                    return Some(Message::AutoEq(AutoEqMessage::ImportFromClipboard));
                }
            }
            if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                return Some(Message::DismissConfirmDialog);
            }
        }
        None
    });

    let file_drop_sub = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Subscription::none()
    } else {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
                Some(Message::Profiles(ProfilesMessage::FileImported(Some(path))))
            } else {
                None
            }
        })
    };

    Subscription::batch(vec![tick_sub, close_sub, keyboard_sub, file_drop_sub])
}
