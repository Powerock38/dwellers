use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    data::{MobId, ObjectId},
    tilemap::TILE_SIZE,
    tilemap_data::TilemapData,
    utils::transform_to_index,
    SpriteLoader, CHUNK_SIZE,
};

const Z_INDEX: f32 = 11.0;

#[derive(Event)]
pub struct SpawnMobsOnChunk(pub IVec2);

pub struct MobData {
    sprite_name: &'static str,
    speed: f32,
    loot: ObjectId,
}

impl MobData {
    pub fn new(sprite_name: &'static str, speed: f32, loot: ObjectId) -> Self {
        MobData {
            sprite_name,
            speed,
            loot,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Mob {
    speed: f32,
    pub loot: ObjectId,
    move_queue: Vec<IVec2>, // next move is at the end
}

#[derive(Bundle)]
pub struct MobBundle {
    mob: Mob,
    sprite: SpriteLoader,
    transform: Transform,
}

impl MobBundle {
    pub fn new(id: MobId, index: IVec2) -> Self {
        MobBundle {
            mob: Mob {
                speed: id.data().speed,
                loot: id.data().loot,
                move_queue: Vec::new(),
            },
            sprite: SpriteLoader {
                texture_path: format!("sprites/{}.png", id.data().sprite_name),
            },
            transform: Transform::from_xyz(
                index.x as f32 * TILE_SIZE,
                index.y as f32 * TILE_SIZE,
                Z_INDEX,
            ),
        }
    }
}

pub fn spawn_mobs(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut ev_spawn: EventReader<SpawnMobsOnChunk>,
) {
    let mut rng = rand::rng();

    for SpawnMobsOnChunk(chunk_index) in ev_spawn.read() {
        let Some(index) = TilemapData::find_from_center_chunk_size(
            TilemapData::local_index_to_global(
                *chunk_index,
                IVec2::new(
                    rng.random_range(0..CHUNK_SIZE as i32),
                    rng.random_range(0..CHUNK_SIZE as i32),
                ),
            ),
            |index| {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let index = index + IVec2::new(dx, dy);

                        let Some(tile) = tilemap_data.get(index) else {
                            return false;
                        };

                        if tile.is_blocking() {
                            return false;
                        }
                    }
                }
                true
            },
        ) else {
            error!("No valid spawn position found for mobs");
            return;
        };

        let nb_sheeps = rng.random_range(1..=7);
        let nb_boars = rng.random_range(1..=5);

        for _ in 0..nb_sheeps {
            commands.spawn(MobBundle::new(MobId::Sheep, index));
        }

        for _ in 0..nb_boars {
            commands.spawn(MobBundle::new(MobId::Boar, index));
        }
    }
}

pub fn update_mobs(mut q_mobs: Query<(&mut Mob, &Transform)>, tilemap_data: Res<TilemapData>) {
    for (mut mob, transform) in &mut q_mobs {
        if !mob.move_queue.is_empty() {
            continue;
        }

        let index = transform_to_index(transform);

        // Wander around
        let mut rng = rand::rng();

        if rng.random_bool(0.2) {
            let directions = tilemap_data.non_blocking_neighbours_pos(index, true);

            if let Some(direction) = directions.choose(&mut rng) {
                mob.move_queue.push(*direction);
            }
        }
    }
}

pub fn update_mobs_movement(
    time: Res<Time>,
    mut q_mobs: Query<(&mut Mob, &mut Transform, &mut Sprite)>,
) {
    for (mut mob, mut transform, mut sprite) in &mut q_mobs {
        // Move to next position in queue

        if let Some(next_move) = mob.move_queue.last() {
            let target = Vec2::new(
                next_move.x as f32 * TILE_SIZE,
                next_move.y as f32 * TILE_SIZE,
            );

            let direction = target - transform.translation.truncate();

            if direction.length() < mob.speed * time.delta_secs() {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                mob.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * mob.speed * time.delta_secs();
                transform.translation.y += dir.y * mob.speed * time.delta_secs();

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}
