use bevy::{prelude::*, sprite::Anchor};

use crate::{Dweller, Task, TaskNeeds};

pub struct ObjectData {
    filename: &'static str,
    blocking: bool,
    slot: ObjectSlot,
}

pub enum ObjectSlot {
    Uncarriable,
    Object,
    Tool,
    Armor,
}

impl ObjectData {
    const fn new(filename: &'static str, blocking: bool, slot: ObjectSlot) -> Self {
        Self {
            filename,
            blocking,
            slot,
        }
    }

    pub const fn passable(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Object)
    }

    pub const fn blocking(filename: &'static str) -> Self {
        Self::new(filename, true, ObjectSlot::Object)
    }

    pub const fn passable_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Uncarriable)
    }

    pub const fn blocking_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, true, ObjectSlot::Uncarriable)
    }

    pub const fn tool(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Tool)
    }

    pub const fn armor(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Armor)
    }

    #[inline]
    pub fn is_carriable(&self) -> bool {
        !matches!(self.slot, ObjectSlot::Uncarriable)
    }

    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    #[inline]
    pub fn filename(&self) -> &'static str {
        self.filename
    }

    #[inline]
    pub fn slot(&self) -> &ObjectSlot {
        &self.slot
    }

    pub fn sprite_path(&self) -> String {
        format!("tiles/objects/{}.png", self.filename)
    }
}

#[derive(Component)]
pub struct DwellerObjectPreview;

#[derive(Component)]
pub struct DwellerToolPreview;

#[derive(Component)]
pub struct DwellerArmorPreview;

pub fn update_dwellers_equipment_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_dwellers: Query<(Entity, &Dweller, &Sprite, &Children), Changed<Dweller>>,
    q_object_previews: Query<&DwellerObjectPreview>,
    q_tool_previews: Query<&DwellerToolPreview>,
    q_armor_previews: Query<&DwellerArmorPreview>,
) {
    for (entity, dweller, sprite, children) in &q_dwellers {
        if let Some(object_preview) = children
            .iter()
            .find(|child| q_object_previews.get(**child).is_ok())
        {
            commands.entity(*object_preview).despawn();
        }

        if let Some(object) = dweller.object {
            commands.entity(entity).with_child((
                DwellerObjectPreview,
                Sprite {
                    image: asset_server.load(object.data().sprite_path()),
                    anchor: Anchor::BottomLeft,
                    flip_x: sprite.flip_x,
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    if sprite.flip_x { 8.0 } else { 0.0 },
                    0.0,
                    1.25,
                ))
                .with_scale(Vec3::splat(0.5)),
            ));
        }

        if let Some(tool_preview) = children
            .iter()
            .find(|child| q_tool_previews.get(**child).is_ok())
        {
            commands.entity(*tool_preview).despawn();
        }

        if let Some(tool) = dweller.tool {
            commands.entity(entity).with_child((
                DwellerToolPreview,
                Sprite {
                    image: asset_server.load(tool.data().sprite_path()),
                    anchor: Anchor::BottomLeft,
                    flip_x: sprite.flip_x,
                    ..default()
                },
                Transform::from_xyz(if sprite.flip_x { 5.0 } else { 19.0 }, 2.0, 1.5)
                    .with_scale(Vec3::new(-0.5, 0.5, 0.5)),
            ));
        }

        if let Some(armor_preview) = children
            .iter()
            .find(|child| q_armor_previews.get(**child).is_ok())
        {
            commands.entity(*armor_preview).despawn();
        }

        if let Some(armor) = dweller.armor {
            commands.entity(entity).with_child((
                DwellerArmorPreview,
                Sprite {
                    image: asset_server.load(armor.data().sprite_path()),
                    anchor: Anchor::BottomLeft,
                    flip_x: sprite.flip_x,
                    ..default()
                },
                Transform::from_translation(Vec3::Z),
            ));
        }
    }
}

#[derive(Component)]
pub struct TaskNeedsObjectPreview;

pub fn update_task_needs_object_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_tasks: Query<(Entity, &Task, Option<&Children>), Changed<Task>>,
    q_object_previews: Query<&TaskNeedsObjectPreview>,
) {
    for (entity, task, children) in &q_tasks {
        if let TaskNeeds::Objects(objects) = &task.needs {
            if let Some(children) = children {
                for child in children {
                    if q_object_previews.get(*child).is_ok() {
                        commands.entity(*child).despawn();
                    }
                }
            }

            commands.entity(entity).with_children(|c| {
                let n = objects.len() as f32;

                for (i, object) in objects.iter().enumerate() {
                    let frac = i as f32 / n;
                    let angle = frac * std::f32::consts::TAU;
                    let position = Vec2::new(angle.cos(), angle.sin()) * 4.0 + 8.0;

                    c.spawn((
                        TaskNeedsObjectPreview,
                        Sprite {
                            image: asset_server.load(object.data().sprite_path()),
                            anchor: Anchor::BottomLeft,
                            color: Color::WHITE.with_alpha(0.5),
                            ..default()
                        },
                        Transform::from_translation(position.extend(frac + 0.1))
                            .with_scale(Vec3::splat(0.25)),
                    ));
                }
            });
        }
    }
}
