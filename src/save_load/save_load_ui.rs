use bevy::prelude::*;

use crate::{LoadGame, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, SAVE_DIR};

#[derive(Bundle)]
pub struct UiWindowBundle {
    ui_window: UiWindow,
    node: NodeBundle,
}

impl Default for UiWindowBundle {
    fn default() -> Self {
        Self {
            ui_window: UiWindow,
            node: NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct UiWindow;

#[derive(Component)]
pub struct LoadButton(pub String);

pub fn spawn_load_save_ui(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_windows: Query<Entity, With<UiWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        if let Some(window) = q_windows.iter().next() {
            commands.entity(window).despawn_recursive();
        } else {
            commands
                .spawn(UiWindowBundle::default())
                .with_children(|c| {
                    if let Ok(save_files) =
                        std::fs::read_dir(format!("assets/{SAVE_DIR}")).map(|dir| {
                            dir.filter_map(|entry| {
                                entry.ok().and_then(|entry| {
                                    if let Some(extension) = entry.path().extension() {
                                        if extension == "bin" {
                                            return entry
                                                .file_name()
                                                .into_string()
                                                .ok()
                                                .map(|file_name| file_name.replacen(".bin", "", 1));
                                        }
                                    }

                                    None
                                })
                            })
                            .collect::<Vec<_>>()
                        })
                    {
                        for save_file in save_files {
                            c.spawn((
                                ButtonBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(5.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    border_color: BorderColor(Color::BLACK),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                LoadButton(save_file.clone()),
                            ))
                            .with_children(|c| {
                                c.spawn(TextBundle::from_section(
                                    save_file,
                                    TextStyle {
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    }
                });
        }
    }
}

pub fn update_save_load_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &LoadButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (load_button, interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;

                commands.insert_resource(LoadGame(load_button.0.clone()));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
