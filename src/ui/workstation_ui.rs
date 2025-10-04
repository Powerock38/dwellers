use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    BG_PRIMARY, Task, TaskKind, TilePlaced, data::WORKSTATIONS, extract_ok,
    TilemapData,
};

#[derive(Event)]
pub struct OpenWorkstationUi {
    pub entity: Entity,
}

#[derive(Component)]
#[require(
    Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        position_type: PositionType::Absolute,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    BackgroundColor(BG_PRIMARY.with_alpha(0.7))
)]
pub struct UiBackground;

#[derive(Component)]
#[require(
    Node {
        width: Val::Px(200.),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(5.0)),
        border: UiRect::all(Val::Px(4.0)),
        ..default()
    },
    BorderColor::all(Color::BLACK),
    BackgroundColor(BG_PRIMARY)
)]
pub struct WorkstationUi(pub Entity, pub u128);

pub fn observe_open_workstation_ui(
    open_ui: On<OpenWorkstationUi>,
    mut commands: Commands,
    q_workstation_ui: Query<Entity, With<WorkstationUi>>,
    q_tasks: Query<&Task>,
) {
    for entity in &q_workstation_ui {
        commands.entity(entity).despawn();
    }

    let entity = open_ui.entity;
    let task = extract_ok!(q_tasks.get(entity));

    if let TaskKind::Workstation { .. } = task.kind {
        debug!("Workstation UI opened: {:?}", task);
        commands
            .spawn(UiBackground)
            .observe(
                |pointer_click: On<Pointer<Click>>, mut commands: Commands| {
                    commands.entity(pointer_click.entity).despawn();
                },
            )
            .with_child(WorkstationUi(
                entity,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            ));
    }
}

//TODO: use a better reactivity system
pub fn update_workstation_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_data: Res<TilemapData>,
    q_workstation_ui: Query<(Entity, &WorkstationUi)>,
    q_tasks: Query<&Task>,
    mut changes: Local<HashMap<u128, u32>>,
) {
    for (ui_entity, workstation_ui) in &q_workstation_ui {
        let entity = workstation_ui.0;

        let Ok(task) = q_tasks.get(entity) else {
            continue;
        };

        let TaskKind::Workstation { amount } = task.kind else {
            continue;
        };

        let Some(TilePlaced {
            object: Some(workstation),
            ..
        }) = tilemap_data.get(task.pos)
        else {
            continue;
        };

        let Some(recipe) = WORKSTATIONS.get(&workstation) else {
            continue;
        };

        let Ok(mut ec) = commands.get_entity(ui_entity) else {
            continue;
        };

        if let Some(old_amount) = changes.get(&workstation_ui.1)
            && *old_amount == amount
        {
            continue;
        }

        changes.insert(workstation_ui.1, amount);

        ec.despawn_related::<Children>().with_children(|c| {
            // Minus button
            c.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_child((Text::new("-"), TextFont::from_font_size(20.0)))
            .observe(
                move |mut pointer_click: On<Pointer<Click>>, mut q_tasks: Query<&mut Task>| {
                    pointer_click.propagate(false);
                    let mut task = extract_ok!(q_tasks.get_mut(entity));
                    if let TaskKind::Workstation { ref mut amount, .. } = task.kind {
                        *amount = amount.saturating_sub(1);
                    }
                },
            );

            c.spawn(ImageNode::new(
                asset_server.load(recipe.0.data().sprite_path()),
            ));
            c.spawn(Text::new(format!("x{amount}")));

            // Plus button
            c.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_child((Text::new("+"), TextFont::from_font_size(20.0)))
            .observe(
                move |mut pointer_click: On<Pointer<Click>>, mut q_tasks: Query<&mut Task>| {
                    pointer_click.propagate(false);
                    let mut task = extract_ok!(q_tasks.get_mut(entity));
                    if let TaskKind::Workstation { ref mut amount, .. } = task.kind {
                        *amount = amount.saturating_add(1);
                    }
                },
            );
        });
    }
}
