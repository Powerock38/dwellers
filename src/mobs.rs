use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    data::ObjectId,
    extract_ok,
    tilemap::{TilemapData, TILE_SIZE},
    SpawnEntitiesOnChunk, SpriteLoaderBundle, CHUNK_SIZE,
};

const Z_INDEX: f32 = 11.0;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Mob {
    speed: f32,
    move_queue: Vec<IVec2>, // next move is at the end
    pub loot: ObjectId,
}

impl Mob {
    pub fn new(speed: f32, loot: ObjectId) -> Self {
        Mob {
            speed,
            move_queue: Vec::new(),
            loot,
        }
    }
}

#[derive(Bundle)]
pub struct MobBundle {
    mob: Mob,
    sprite: SpriteLoaderBundle,
}

impl MobBundle {
    pub fn new(mob: Mob, texture_path: &str, position: Vec2) -> Self {
        MobBundle {
            mob,
            sprite: SpriteLoaderBundle::new(texture_path, position.x, position.y, Z_INDEX),
        }
    }
}

pub fn spawn_mobs(
    mut commands: Commands,
    q_tilemap: Query<&TilemapData>,
    mut ev_spawn_entities_on_chunk: EventReader<SpawnEntitiesOnChunk>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    for SpawnEntitiesOnChunk(chunk_index) in ev_spawn_entities_on_chunk.read() {
        let Some(spawn_pos) =
            TilemapData::find_from_center(IVec2::splat(CHUNK_SIZE as i32 / 2), |index| {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let index = TilemapData::local_index_to_global(
                            *chunk_index,
                            index + IVec2::new(dx, dy),
                        );

                        let Some(tile_data) = tilemap_data.get(index) else {
                            return false;
                        };

                        if tile_data.is_blocking() {
                            return false;
                        }
                    }
                }
                true
            })
        else {
            error!("No valid spawn position found for mobs");
            return;
        };

        let nb_sheeps = 5;
        let nb_boars = 3;

        for _ in 0..nb_sheeps {
            commands.spawn(MobBundle::new(
                Mob::new(60.0, ObjectId::Rug),
                "sprites/sheep.png",
                Vec2::new(
                    spawn_pos.x as f32 * TILE_SIZE,
                    spawn_pos.y as f32 * TILE_SIZE,
                ),
            ));
        }

        for _ in 0..nb_boars {
            commands.spawn(MobBundle::new(
                Mob::new(50.0, ObjectId::Rug),
                "sprites/boar.png",
                Vec2::new(
                    spawn_pos.x as f32 * TILE_SIZE,
                    spawn_pos.y as f32 * TILE_SIZE,
                ),
            ));
        }
    }
}

pub fn update_mobs(mut q_mobs: Query<(&mut Mob, &Transform)>, mut q_tilemap: Query<&TilemapData>) {
    let tilemap_data = extract_ok!(q_tilemap.get_single_mut());

    for (mut mob, transform) in &mut q_mobs {
        if !mob.move_queue.is_empty() {
            continue;
        }

        let index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        // Wander around
        let mut rng = rand::thread_rng();

        let directions = tilemap_data.non_blocking_neighbours_pos(index, true);

        if let Some(direction) = directions.choose(&mut rng) {
            mob.move_queue.push(*direction);
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

            if direction.length() < mob.speed * time.delta_seconds() {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                mob.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * mob.speed * time.delta_seconds();
                transform.translation.y += dir.y * mob.speed * time.delta_seconds();

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}
