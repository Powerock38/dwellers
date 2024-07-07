use bevy::prelude::*;

use crate::{
    actions::{ActionKind, CurrentAction},
    GameState, LoadGame, SaveGame,
};

mod actions_ui;
pub use actions_ui::*;

pub fn init_font(asset_server: Res<AssetServer>, mut query: Query<&mut Text, Added<Text>>) {
    for mut text in &mut query {
        for section in &mut text.sections {
            section.style.font = asset_server.load("alagard.ttf");
        }
    }
}

#[derive(Component)]
pub enum UiButton {
    Action(ActionKind),
    SaveGame,
    LoadGame(String),
}

impl UiButton {
    pub const NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
    pub const HOVERED: Color = Color::srgb(0.25, 0.25, 0.25);
    pub const PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);
}

pub fn build_ui_button(c: &mut ChildBuilder, ui_button: UiButton, text: impl Into<String>) {
    c.spawn((
        ui_button,
        ButtonBundle {
            style: Style {
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: UiButton::NORMAL.into(),
            ..default()
        },
    ))
    .with_children(|c| {
        c.spawn(TextBundle::from_section(
            text,
            TextStyle {
                font_size: 20.0,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
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
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.5).into(),
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
                    border_color.0 = bevy::color::palettes::css::RED.into();

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
                    border_color.0 = bevy::color::palettes::css::RED.into();

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
