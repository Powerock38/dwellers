use bevy::prelude::*;

use crate::{build_ui_button, GameState, SaveName, UiButton, UiWindow, SAVE_DIR};

pub fn spawn_load_save_ui(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_windows: Query<Entity, With<UiWindow>>,
    save_name: Res<SaveName>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        if let Some(window) = q_windows.iter().next() {
            commands.entity(window).despawn_recursive();
            next_state.set(GameState::Running);
        } else {
            commands.spawn(UiWindow).with_children(|c| {
                // Save button
                build_ui_button(c, UiButton::SaveGame, format!("Save {}", save_name.0));

                // Saves list
                if let Ok(save_files) = std::fs::read_dir(format!("assets/{SAVE_DIR}")).map(|dir| {
                    let mut saves = dir
                        .filter_map(|entry| {
                            entry
                                .ok()
                                .filter(|entry| {
                                    entry.file_type().ok().is_some_and(|ft| ft.is_dir())
                                })
                                .and_then(|entry| entry.file_name().into_string().ok())
                        })
                        .collect::<Vec<_>>();

                    saves.sort_by(|a, b| b.cmp(a));
                    saves
                }) {
                    for save_file in save_files {
                        build_ui_button(c, UiButton::LoadGame(save_file.clone()), save_file);
                    }
                } else {
                    error!("Failed to read save files");
                }
            });

            next_state.set(GameState::Paused);
        }
    }
}
