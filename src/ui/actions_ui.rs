use bevy::prelude::*;

use crate::{build_ui_button, data::BUILD_RECIPES, ActionKind, TaskKind, TaskNeeds, UiButton};

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
                    build_ui_button(
                        c,
                        UiButton::Action(ActionKind::TaskWithNeeds(
                            TaskKind::Build { result: *result },
                            TaskNeeds::Objects(cost.to_vec()),
                        )),
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
                build_button(c, ActionKind::Task(TaskKind::GoThere));

                build_button(c, ActionKind::Cancel);
            });
        });
}

fn build_button(c: &mut ChildBuilder, kind: ActionKind) {
    let text = kind.to_string();
    build_ui_button(c, UiButton::Action(kind), text);
}
