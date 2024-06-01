use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::*;

use crate::{
    extract_ok,
    terrain::{find_from_center, TilemapData, TERRAIN_SIZE, TILE_SIZE},
    tiles::{ObjectData, TileData},
};

const Z_INDEX: f32 = 11.0;

#[derive(Component)]
pub struct Mob {
    speed: f32,
    move_queue: Vec<IVec2>, // next move is at the end
    pub loot: ObjectData,
}

impl Mob {
    pub fn new(speed: f32, loot: ObjectData) -> Self {
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
    sprite: SpriteBundle,
}

impl MobBundle {
    pub fn new(mob: Mob, texture: Handle<Image>, position: Vec2) -> Self {
        MobBundle {
            mob,
            sprite: SpriteBundle {
                texture,
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, Z_INDEX),
                ..default()
            },
        }
    }
}

pub fn spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_tilemap: Query<&TilemapData>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    let Some(spawn_pos) = find_from_center(IVec2::splat(TERRAIN_SIZE as i32 / 2), |index| {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if tilemap_data.0.get(index + IVec2::new(dx, dy)) != Some(TileData::GRASS_FLOOR) {
                    return false;
                }
            }
        }
        true
    }) else {
        println!("No valid spawn position found for mobs");
        return;
    };

    let nb_sheeps = 5;
    let nb_boars = 3;

    for _ in 0..nb_sheeps {
        commands.spawn(MobBundle::new(
            Mob::new(60.0, ObjectData::RUG),
            asset_server.load("sprites/sheep.png"),
            Vec2::new(
                spawn_pos.x as f32 * TILE_SIZE,
                spawn_pos.y as f32 * TILE_SIZE,
            ),
        ));
    }

    for _ in 0..nb_boars {
        commands.spawn(MobBundle::new(
            Mob::new(50.0, ObjectData::RUG),
            asset_server.load("sprites/boar.png"),
            Vec2::new(
                spawn_pos.x as f32 * TILE_SIZE,
                spawn_pos.y as f32 * TILE_SIZE,
            ),
        ));
    }
}

pub fn update_mobs(mut q_mobs: Query<(&mut Mob, &Transform)>, mut q_tilemap: Query<&TilemapData>) {
    let tilemap_data = extract_ok!(q_tilemap.get_single_mut());

    for (mut mob, transform) in &mut q_mobs {
        if !mob.move_queue.is_empty() {
            continue;
        }

        let mut index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        // Wander around
        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.5) {
            index.x += rng.gen_range(-1..=1);
        } else {
            index.y += rng.gen_range(-1..=1);
        }

        let Some(tiledata) = tilemap_data.0.get(index) else {
            continue;
        };

        if !tiledata.is_blocking() {
            mob.move_queue.push(index);
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
