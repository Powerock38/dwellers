use bevy::prelude::*;

use crate::{
    data::BUILD_RECIPES, extract_ok, ActionKind, Dweller, DwellersSelected, TaskKind, TaskNeeds,
    UiButton,
};

#[derive(Component)]
pub struct DwellersSelectedUi;

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
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|c| {
                for (name, result, cost) in BUILD_RECIPES {
                    c.spawn(UiButton::Action(ActionKind::TaskWithNeeds(
                        TaskKind::Build { result: *result },
                        TaskNeeds::Objects(cost.to_vec()),
                    )))
                    .with_child(Text::new(*name))
                    .with_child(ImageNode::new(asset_server.load(result.sprite_path())));
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
                    TaskKind::Hunt,
                    TaskKind::Pickup,
                    TaskKind::Stockpile,
                    TaskKind::Smoothen,
                    TaskKind::Walk,
                ] {
                    c.spawn(UiButton::Action(ActionKind::Task(task_kind)))
                        .with_child(Text::new(
                            format!("{task_kind:?}").split_whitespace().next().unwrap(),
                        ))
                        .with_child(ImageNode::new(asset_server.load(task_kind.sprite_path())));
                }

                c.spawn(UiButton::Action(ActionKind::Cancel))
                    .with_child(Text::new("Cancel"));
            });
        });
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
