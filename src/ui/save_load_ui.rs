use bevy::prelude::*;

use crate::{
    GameState, LoadGame, SAVE_DIR, SaveChunk, SaveName, TilemapData, UiButton, UiWindow,
    locale::Locale,
    save_load::SaveResources,
};

pub fn spawn_load_save_ui(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_windows: Query<Entity, With<UiWindow>>,
    save_name: Res<SaveName>,
    locale: Res<Locale>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        if let Some(window) = q_windows.iter().next() {
            commands.entity(window).despawn();
            next_state.set(GameState::Running);
        } else {
            commands.spawn(UiWindow).with_children(|c| {
                // Кнопка сохранения
                let save_label = format!("{} {}", locale.t("ui.save"), save_name.0);
                c.spawn(UiButton)
                    .with_child(Text::new(save_label))
                    .observe(
                        |_: On<Pointer<Click>>,
                         mut commands: Commands,
                         tilemap_data: Res<TilemapData>,
                         q_windows: Query<Entity, With<UiWindow>>| {
                            for chunk_pos in tilemap_data.chunks.keys() {
                                commands.write_message(SaveChunk(*chunk_pos, false));
                            }
                            commands.trigger(SaveResources);

                            if let Some(window) = q_windows.iter().next() {
                                commands.entity(window).despawn();
                            }
                        },
                    );

                // Список сохранений
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
                    let load_prefix = locale.t("ui.load");
                    for save_file in save_files {
                        let load_label = format!("{} {save_file}", load_prefix);
                        c.spawn(UiButton)
                            .with_child(Text::new(load_label))
                            .observe(
                                move |_: On<Pointer<Click>>,
                                mut commands: Commands,
                                q_windows: Query<Entity, With<UiWindow>>| {
                                    commands.trigger(LoadGame(save_file.clone()));

                                    if let Some(window) = q_windows.iter().next() {
                                        commands.entity(window).despawn();
                                    }
                                },
                            );
                    }
                } else {
                    error!("Не удалось прочитать файлы сохранений");
                }
            });

            next_state.set(GameState::Paused);
        }
    }
}
