use bevy::prelude::*;

use crate::{
    actions::{ActionKind, CurrentAction},
    tasks::{BuildResult, TaskKind},
    tiles::{ObjectData, TileData},
};

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct ActionButton(pub ActionKind);

pub fn update_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &ActionButton,
            &Interaction,
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    current_action: Option<Res<CurrentAction>>,
    mut current_action_existed: Local<bool>,
) {
    for (action_button, interaction, interaction_changed, mut color, mut border_color) in
        &mut interaction_query
    {
        if interaction_changed.is_changed() {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    border_color.0 = Color::RED;

                    if current_action
                        .as_ref()
                        .is_some_and(|action| action.kind == action_button.0)
                    {
                        commands.remove_resource::<CurrentAction>();
                    } else {
                        commands.insert_resource(CurrentAction::new(action_button.0));
                    }

                    continue;
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                    border_color.0 = Color::WHITE;

                    continue;
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                    border_color.0 = Color::BLACK;
                }
            }
        }

        if let Some(current_action) = &current_action {
            *current_action_existed = true;

            if current_action.kind == action_button.0 {
                border_color.0 = Color::RED;

                continue;
            }
        } else {
            *current_action_existed = false;
        }

        if color.0 != HOVERED_BUTTON {
            *color = NORMAL_BUTTON.into();
            border_color.0 = Color::BLACK;
        }
    }
}

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|c| {
            c.spawn(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                for (name, result, cost) in [
                    (
                        "wall",
                        BuildResult::Tile(TileData::DUNGEON_WALL),
                        ObjectData::ROCK,
                    ),
                    (
                        "table",
                        BuildResult::Object(ObjectData::TABLE),
                        ObjectData::WOOD,
                    ),
                    (
                        "stool",
                        BuildResult::Object(ObjectData::STOOL),
                        ObjectData::WOOD,
                    ),
                    (
                        "bed",
                        BuildResult::Object(ObjectData::BED),
                        ObjectData::WOOD,
                    ),
                    (
                        "door",
                        BuildResult::Object(ObjectData::DOOR),
                        ObjectData::WOOD,
                    ),
                ] {
                    build_button_text(
                        c,
                        ActionKind::Task(TaskKind::Build { result, cost }),
                        format!("Build {name}"),
                    );
                }
            });

            c.spawn(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                build_button(c, ActionKind::Task(TaskKind::Dig));
                build_button(c, ActionKind::Task(TaskKind::Smoothen));
                build_button(c, ActionKind::Task(TaskKind::Chop));
                build_button(c, ActionKind::Task(TaskKind::Bridge));
                build_button(c, ActionKind::Task(TaskKind::Hunt));
                build_button(c, ActionKind::Task(TaskKind::Pickup));
                build_button(c, ActionKind::Task(TaskKind::Stockpile));

                build_button(c, ActionKind::Cancel);
            });
        });
}

fn build_button(c: &mut ChildBuilder, kind: ActionKind) {
    build_button_text(c, kind, kind.to_string());
}

fn build_button_text(c: &mut ChildBuilder, kind: ActionKind, text: String) {
    c.spawn((
        ActionButton(kind),
        ButtonBundle {
            style: Style {
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
    ))
    .with_children(|c| {
        c.spawn(TextBundle::from_section(
            text,
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
}
