use bevy::prelude::*;

use crate::{
    actions::{ActionKind, CurrentAction},
    GameState, LoadGame, SaveGame,
};

mod actions_ui;
pub use actions_ui::*;

#[derive(Component)]
pub enum UiButton {
    Action(ActionKind),
    SaveGame,
    LoadGame(String),
}

impl UiButton {
    pub const NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
    pub const HOVERED: Color = Color::rgb(0.25, 0.25, 0.25);
    pub const PRESSED: Color = Color::rgb(0.35, 0.75, 0.35);
}

#[derive(Component)]
pub struct UiWindow;

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

pub fn update_ui_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &UiButton,
            &Interaction,
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    q_windows: Query<Entity, With<UiWindow>>,
    current_action: Option<Res<CurrentAction>>,
    mut current_action_existed: Local<bool>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (ui_button, interaction, interaction_changed, mut color, mut border_color) in
        &mut interaction_query
    {
        if interaction_changed.is_changed() {
            match *interaction {
                Interaction::Pressed => {
                    *color = UiButton::PRESSED.into();
                    border_color.0 = Color::RED;

                    match ui_button {
                        UiButton::Action(action) => {
                            if current_action
                                .as_ref()
                                .is_some_and(|current_action| current_action.kind == *action)
                            {
                                commands.remove_resource::<CurrentAction>();
                            } else {
                                commands.insert_resource(CurrentAction::new(action.clone()));
                            }

                            continue;
                        }

                        UiButton::LoadGame(save_file) => {
                            commands.insert_resource(LoadGame(save_file.clone()));

                            if let Some(window) = q_windows.iter().next() {
                                commands.entity(window).despawn_recursive();
                            }
                        }

                        UiButton::SaveGame => {
                            commands.insert_resource(SaveGame);

                            if let Some(window) = q_windows.iter().next() {
                                commands.entity(window).despawn_recursive();
                            }

                            next_state.set(GameState::Running);
                        }
                    }
                }
                Interaction::Hovered => {
                    *color = UiButton::HOVERED.into();
                    border_color.0 = Color::WHITE;

                    continue;
                }
                Interaction::None => {
                    *color = UiButton::NORMAL.into();
                    border_color.0 = Color::BLACK;
                }
            }
        }

        if let UiButton::Action(action) = ui_button {
            if let Some(current_action) = &current_action {
                *current_action_existed = true;

                if current_action.kind == *action {
                    border_color.0 = Color::RED;

                    continue;
                }
            } else {
                *current_action_existed = false;
            }
        }

        if color.0 != UiButton::HOVERED {
            *color = UiButton::NORMAL.into();
            border_color.0 = Color::BLACK;
        }
    }
}
