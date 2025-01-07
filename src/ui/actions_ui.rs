use bevy::prelude::*;

use crate::{
    build_ui_button, data::BUILD_RECIPES, extract_ok, ActionKind, Dweller, DwellersSelected,
    TaskKind, TaskNeeds, UiButton,
};

#[derive(Component)]
pub struct DwellersSelectedUi;

pub fn spawn_ui(mut commands: Commands) {
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

            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
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
                build_button(c, ActionKind::Task(TaskKind::Walk));

                build_button(c, ActionKind::Cancel);
            });
        });
}

fn build_button(c: &mut ChildBuilder, kind: ActionKind) {
    let text = kind.to_string();
    build_ui_button(c, UiButton::Action(kind), text);
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
