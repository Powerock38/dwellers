use bevy::prelude::*;

use crate::{data::BUILD_RECIPES, ActionKind, TaskKind, TaskNeeds, UiButton};

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
                for (name, result, cost) in BUILD_RECIPES {
                    build_button_text(
                        c,
                        ActionKind::TaskWithNeeds(
                            TaskKind::Build { result: *result },
                            TaskNeeds::Objects(cost.to_vec()),
                        ),
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
                build_button(c, ActionKind::Task(TaskKind::Harvest));
                build_button(c, ActionKind::Task(TaskKind::Bridge));
                build_button(c, ActionKind::Task(TaskKind::Hunt));
                build_button(c, ActionKind::Task(TaskKind::Pickup));
                build_button(c, ActionKind::Task(TaskKind::Stockpile));

                build_button(c, ActionKind::Cancel);
            });
        });
}

fn build_button(c: &mut ChildBuilder, kind: ActionKind) {
    let text = kind.to_string();
    build_button_text(c, kind, text);
}

fn build_button_text(c: &mut ChildBuilder, kind: ActionKind, text: String) {
    c.spawn((
        UiButton::Action(kind),
        ButtonBundle {
            style: Style {
                border: UiRect::all(Val::Px(5.0)),
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
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
}
