use bevy::prelude::*;

use crate::{GameState, LoadGame, SaveGame, SaveName, UiButton, UiWindow, SAVE_DIR};

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
                c.spawn(UiButton)
                    .with_child(Text::new(format!("Save {}", save_name.0)))
                    .observe(
                        |_: Trigger<Pointer<Click>>,
                         mut commands: Commands,
                         q_windows: Query<Entity, With<UiWindow>>,
                         mut next_state: ResMut<NextState<GameState>>| {
                            commands.insert_resource(SaveGame);

                            if let Some(window) = q_windows.iter().next() {
                                commands.entity(window).despawn_recursive();
                            }

                            next_state.set(GameState::Running);
                        },
                    );

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
                        c.spawn(UiButton)
                            .with_child(Text::new(format!("Load {save_file}")))
                            .observe(
                                move |_: Trigger<Pointer<Click>>,
                                mut commands: Commands,
                                q_windows: Query<Entity, With<UiWindow>>| {
                                    commands.insert_resource(LoadGame(save_file.clone()));

                                    if let Some(window) = q_windows.iter().next() {
                                        commands.entity(window).despawn_recursive();
                                    }
                                },
                            );
                    }
                } else {
                    error!("Failed to read save files");
                }
            });

            next_state.set(GameState::Paused);
        }
    }
}
