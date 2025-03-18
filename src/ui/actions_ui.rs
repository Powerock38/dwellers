use bevy::prelude::*;

use crate::{
    actions::CurrentAction, data::BUILD_RECIPES, extract_ok, utils::pascal_case_to_title_case,
    ActionKind, Dweller, DwellersSelected, TaskKind, TaskNeeds, UiButton,
};

#[derive(Component)]
pub struct DwellersSelectedUi;

#[derive(Component)]
pub struct CoordinatesUi;

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                DwellersSelectedUi,
                Text::new(""),
                BackgroundColor(Color::BLACK.with_alpha(0.5)),
            ));

            c.spawn((
                CoordinatesUi,
                Text::new(""),
                BackgroundColor(Color::BLACK.with_alpha(0.5)),
            ));

            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|c| {
                for (result, cost) in BUILD_RECIPES {
                    c.spawn(UiButton)
                        .with_child(Text::new(pascal_case_to_title_case(&result.debug_name())))
                        .with_child(ImageNode::new(asset_server.load(result.sprite_path())))
                        .observe(get_observer_action_button(ActionKind::TaskWithNeeds(
                            TaskKind::Build { result: *result },
                            TaskNeeds::Objects(cost.to_vec()),
                        )));
                }
            });

            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|c| {
                for task_kind in [
                    TaskKind::Dig,
                    TaskKind::Harvest,
                    TaskKind::Attack,
                    TaskKind::Fish,
                    TaskKind::Pickup,
                    TaskKind::Stockpile,
                    TaskKind::Smoothen,
                    TaskKind::Walk,
                ] {
                    c.spawn(UiButton)
                        .with_child(Text::new(pascal_case_to_title_case(
                            format!("{task_kind:?}").split_whitespace().next().unwrap(),
                        )))
                        .with_child(ImageNode::new(asset_server.load(task_kind.sprite_path())))
                        .observe(get_observer_action_button(ActionKind::Task(task_kind)));
                }

                c.spawn(UiButton)
                    .with_child(Text::new("Cancel"))
                    .observe(get_observer_action_button(ActionKind::Cancel));
            });
        });
}

pub fn get_observer_action_button(
    action: ActionKind,
) -> impl FnMut(
    Trigger<Pointer<Click>>,
    Commands,
    Res<CurrentAction>,
    Query<&mut BorderColor, With<UiButton>>,
) {
    move |trigger: _, mut commands: _, current_action: _, mut q_borders: _| {
        if current_action.kind == action {
            commands.insert_resource(CurrentAction::default());
        } else {
            commands.insert_resource(CurrentAction::new(action.clone()));
        }

        for mut border in &mut q_borders {
            border.0 = Color::BLACK;
        }

        if let Ok(mut border) = q_borders.get_mut(trigger.entity()) {
            border.0 = bevy::color::palettes::css::RED.into();
        }
    }
}

pub fn update_dwellers_selected(
    dwellers_selected: Res<DwellersSelected>,
    q_dwellers: Query<&Dweller>,
    mut q_dwellers_selected_ui: Query<&mut Text, With<DwellersSelectedUi>>,
) {
    if dwellers_selected.is_changed() {
        let mut dwellers_selected_ui = extract_ok!(q_dwellers_selected_ui.get_single_mut());

        dwellers_selected_ui.0 = dwellers_selected
            .list()
            .iter()
            .filter_map(|e| q_dwellers.get(*e).ok().map(|d| d.name.clone()))
            .collect::<Vec<String>>()
            .join(", ");
    }
}
