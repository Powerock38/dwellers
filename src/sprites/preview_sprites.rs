use bevy::{platform::collections::HashMap, prelude::*, sprite::Anchor};

use crate::{Dweller, TILE_SIZE, Task, TaskKind, TaskNeeds};

#[derive(Component)]
pub enum DwellerEquipmentPreview {
    Object,
    Tool,
    Armor,
}

pub fn update_dwellers_equipment_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_dwellers: Query<(Entity, &Dweller, &Sprite, &Children), Changed<Dweller>>,
    q_previews: Query<&DwellerEquipmentPreview>,
) {
    for (entity, dweller, sprite, children) in &q_dwellers {
        for preview in children
            .iter()
            .filter(|child| q_previews.get(*child).is_ok())
        {
            commands.entity(preview).despawn();
        }

        for (preview_variant, equipment, transform) in [
            (
                DwellerEquipmentPreview::Object,
                dweller.object,
                Transform::from_xyz(if sprite.flip_x { TILE_SIZE / 2.0 } else { 0.0 }, 0.0, 1.25)
                    .with_scale(Vec3::splat(0.5)),
            ),
            (
                DwellerEquipmentPreview::Tool,
                dweller.tool,
                Transform::from_xyz(if sprite.flip_x { 5.0 } else { 19.0 }, 2.0, 1.5)
                    .with_scale(Vec3::new(-0.5, 0.5, 0.5)),
            ),
            (
                DwellerEquipmentPreview::Armor,
                dweller.armor,
                Transform::from_translation(Vec3::Z),
            ),
        ] {
            if let Some(equipment) = equipment {
                commands.entity(entity).with_child((
                    preview_variant,
                    Sprite {
                        image: asset_server.load(equipment.data().sprite_path()),
                        flip_x: sprite.flip_x,
                        ..default()
                    },
                    Anchor::BOTTOM_LEFT,
                    transform,
                ));
            }
        }
    }
}

#[derive(Component)]
pub struct TaskNeedsPreview;

pub fn update_task_needs_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    q_tasks: Query<(Entity, &TaskNeeds, Option<&Children>), Changed<TaskNeeds>>,
    mut q_object_previews: Query<&mut Sprite, With<TaskNeedsPreview>>,
) {
    const TASK_OBJECT_PREVIEW_SCALE: f32 = 0.25;

    for (entity, task_needs, children) in &q_tasks {
        if let TaskNeeds::Objects(objects) = task_needs {
            if let Some(children) = children {
                for child in children {
                    if q_object_previews.get(*child).is_ok() {
                        commands.entity(*child).despawn();
                    }
                }
            }

            commands.entity(entity).with_children(|c| {
                let n = objects.len();

                for (i, object) in objects.iter().enumerate() {
                    let frac = i as f32 / n as f32;
                    let angle = frac * std::f32::consts::TAU;
                    let position = if n == 1 {
                        Vec2::splat(TILE_SIZE / 2.0)
                    } else {
                        Vec2::new(angle.cos(), angle.sin()) * 4.0 + TILE_SIZE / 2.0
                    } - Vec2::splat(TILE_SIZE * TASK_OBJECT_PREVIEW_SCALE / 2.0);

                    c.spawn((
                        TaskNeedsPreview,
                        Sprite {
                            image: asset_server.load(object.data().sprite_path()),
                            color: Color::WHITE.with_alpha(0.5),
                            ..default()
                        },
                        Anchor::BOTTOM_LEFT,
                        Transform::from_translation(position.extend(frac + 1.0))
                            .with_scale(Vec3::splat(TASK_OBJECT_PREVIEW_SCALE)),
                    ));
                }
            });
        }
    }

    // Sprite blinking
    for mut sprite in &mut q_object_previews {
        sprite
            .color
            .set_alpha((0.5 + (time.elapsed_secs_wrapped() * 4.0).sin()).clamp(0.25, 0.75));
    }
}

#[derive(Component)]
pub struct TaskBuildPreview;

pub fn update_task_build_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_tasks: Query<(Entity, &Task, Option<&Children>), Added<Task>>,
    q_build_previews: Query<(), With<TaskBuildPreview>>,
) {
    for (entity, task, children) in &q_tasks {
        match task.kind {
            // Build result preview
            TaskKind::Build { result } => {
                if let Some(children) = children {
                    for child in children {
                        if q_build_previews.get(*child).is_ok() {
                            commands.entity(*child).despawn();
                        }
                    }
                }

                commands.entity(entity).with_child((
                    TaskBuildPreview,
                    Sprite {
                        image: asset_server.load(result.sprite_path()),
                        color: Color::WHITE.with_alpha(0.5),
                        ..default()
                    },
                    Anchor::BOTTOM_LEFT,
                    Transform::from_translation(Vec3::NEG_Z),
                ));
            }

            _ => {}
        }
    }
}

#[derive(Component)]
pub struct TaskWorkstationPreview;

pub fn update_task_workstation_preview(
    mut commands: Commands,
    q_tasks: Query<(Entity, &Task, Option<&Children>), Changed<Task>>,
    q_workstation_previews: Query<(), With<TaskWorkstationPreview>>,
    mut changes: Local<HashMap<Entity, u32>>,
) {
    for (entity, task, children) in &q_tasks {
        match task.kind {
            // Workstation amount preview
            TaskKind::Workstation { amount } => {
                if let Some(old_amount) = changes.get(&entity)
                    && *old_amount == amount
                {
                    continue;
                }

                changes.insert(entity, amount);

                if let Some(children) = children {
                    for child in children {
                        if q_workstation_previews.get(*child).is_ok() {
                            commands.entity(*child).despawn();
                        }
                    }
                }

                commands.entity(entity).with_child((
                    TaskWorkstationPreview,
                    Text2d::new(format!("{amount}")),
                    Anchor::TOP_LEFT,
                    Transform::from_xyz(1., TILE_SIZE, 1.0).with_scale(Vec3::splat(0.25)),
                ));
            }

            _ => {}
        }
    }

    changes.retain(|entity, _| commands.get_entity(*entity).is_ok());
}

#[derive(Component)]
pub struct DwellerNeedsPreview;

pub fn observe_dweller_hover(
    pointer_over: On<Pointer<Over>>,
    mut commands: Commands,
    q_dwellers: Query<&Dweller>,
) {
    let Ok(dweller) = q_dwellers.get(pointer_over.entity) else {
        return;
    };

    commands.entity(pointer_over.entity).with_child((
        DwellerNeedsPreview,
        Text2d::new(format!(
            "<3 {}\n><> {}\nzZ {}",
            dweller.health, dweller.food, dweller.sleep
        )),
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(8., 26., 1.0).with_scale(Vec3::splat(0.25)),
    ));
}

pub fn despawn_dweller_hover(
    pointer_out: On<Pointer<Out>>,
    mut commands: Commands,
    q_dwellers: Query<&Children, With<Dweller>>,
    q_dwellers_hover: Query<(), With<DwellerNeedsPreview>>,
) {
    let Ok(children) = q_dwellers.get(pointer_out.entity) else {
        return;
    };

    for child in children {
        if q_dwellers_hover.get(*child).is_ok() {
            commands.entity(*child).despawn();
        }
    }
}
