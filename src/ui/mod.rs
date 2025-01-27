use bevy::prelude::*;

use crate::{
    actions::{ActionKind, CurrentAction},
    GameState, LoadGame, SaveGame,
};

mod actions_ui;
pub use actions_ui::*;
mod workstation_ui;
pub use workstation_ui::*;

pub fn init_font(asset_server: Res<AssetServer>, mut query: Query<&mut TextFont, Added<TextFont>>) {
    for mut font in &mut query {
        font.font = asset_server.load("alagard.ttf");
    }
}

#[derive(Component)]
#[require(
    Button,
    Node(|| Node {
        padding: UiRect::all(Val::Px(5.0)),
        border: UiRect::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(5.0),
        ..default()
    }),
    BorderColor(|| BorderColor(Color::BLACK)),
    BackgroundColor(|| BackgroundColor(BG_PRIMARY))
)]
pub enum UiButton {
    Action(ActionKind),
    SaveGame,
    LoadGame(String),
}

pub const BG_PRIMARY: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BG_SECONDARY: Color = Color::srgb(0.25, 0.25, 0.25);
pub const BG_TERTIARY: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn build_ui_button(c: &mut ChildBuilder, ui_button: UiButton, text: impl Into<String>) {
    c.spawn(ui_button)
        .with_child((Text::new(text), TextFont::from_font_size(20.0)));
}

#[derive(Component)]
#[require(
    Node(|| Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(10.)),
        ..default()
    }),
    BackgroundColor(|| Color::srgba(0.1, 0.1, 0.1, 0.5))
)]
pub struct UiWindow;

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
                    *color = BG_TERTIARY.into();
                    border_color.0 = bevy::color::palettes::css::RED.into();

                    match ui_button {
                        UiButton::Action(action) => {
                            if current_action
                                .as_ref()
                                .is_some_and(|current_action| current_action.kind == *action)
                            {
                                commands.insert_resource(CurrentAction::default());
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
                    *color = BG_SECONDARY.into();
                    border_color.0 = Color::WHITE;

                    continue;
                }
                Interaction::None => {
                    *color = BG_PRIMARY.into();
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

        if color.0 != BG_SECONDARY {
            *color = BG_PRIMARY.into();
            border_color.0 = Color::BLACK;
        }
    }
}
