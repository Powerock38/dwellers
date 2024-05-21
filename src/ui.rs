use bevy::prelude::*;

use crate::{actions::CurrentAction, tasks::TaskKind};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct ActionButton(pub TaskKind);

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
                        .is_some_and(|action| action.task_kind == action_button.0)
                    {
                        commands.remove_resource::<CurrentAction>();
                    } else {
                        commands.insert_resource(CurrentAction::new(action_button.0));
                    }
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

        if current_action
            .as_ref()
            .is_some_and(|action| action.task_kind == action_button.0)
        {
            *color = PRESSED_BUTTON.into();
            border_color.0 = Color::RED;
            //FIXME: not cleared when current_action is changed / removed
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
                build_button(c, TaskKind::Dig);
                build_button(c, TaskKind::Smoothen);
            });
        });
}

fn build_button(c: &mut ChildBuilder, task_kind: TaskKind) {
    c.spawn((
        ActionButton(task_kind),
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
            task_kind.to_string(),
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
}
