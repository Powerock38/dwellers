use bevy::prelude::*;
use pathfinding::directed::astar::astar;
use rand::prelude::*;

use crate::{
    CHUNK_SIZE, SaveScoped, SpriteLoader, TILE_SIZE, TilemapData,
    data::{MobId, ObjectId},
    dwellers::Dweller,
    sprites::TakingDamage,
    utils::transform_to_pos,
};

const Z_INDEX: f32 = 11.0;
const HOSTILE_MOBS_DETECTION_TILE_RADIUS: i32 = 20;

#[derive(Message)]
pub struct SpawnMobsOnChunk(pub IVec2);

pub struct MobData {
    filename: &'static str,
    speed: f32,
    pub loot: ObjectId,
    health: u32,
    attack: u32,
}

impl MobData {
    pub fn new(
        filename: &'static str,
        health: u32,
        speed: f32,
        attack: u32,
        loot: ObjectId,
    ) -> Self {
        MobData {
            filename,
            speed,
            loot,
            health,
            attack,
        }
    }

    pub fn sprite_path(&self) -> String {
        format!("sprites/{}.png", self.filename)
    }

    pub fn is_hostile(&self) -> bool {
        self.attack > 0
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Name::new("mob"), SaveScoped)]
pub struct Mob {
    pub id: MobId,
    move_queue: Vec<IVec2>, // next move is at the end
    pub health: u32,
}

impl Mob {
    pub fn new(id: MobId) -> Self {
        Mob {
            id,
            move_queue: Vec::new(),
            health: id.data().health,
        }
    }

    pub fn pathfind(&mut self, pos: IVec2, tilemap_data: &TilemapData) {
        let path = astar(
            &pos,
            tilemap_data.astar_successors(),
            |p| (p.x - pos.x).abs() + (p.y - pos.y).abs(),
            |p| *p == pos,
        )
        .map(|(mut path, _)| {
            path.reverse();
            path
        });

        if let Some(path) = path {
            self.move_queue = path;
        }
    }

    pub fn health(&mut self, x: i32) {
        self.health = self
            .health
            .saturating_add_signed(x)
            .min(self.id.data().health);
    }
}

#[derive(Bundle)]
pub struct MobBundle {
    mob: Mob,
    sprite: SpriteLoader,
    transform: Transform,
}

impl MobBundle {
    pub fn new(id: MobId, pos: IVec2) -> Self {
        MobBundle {
            mob: Mob::new(id),
            sprite: SpriteLoader {
                texture_path: id.data().sprite_path(),
            },
            transform: Transform::from_xyz(
                pos.x as f32 * TILE_SIZE,
                pos.y as f32 * TILE_SIZE,
                Z_INDEX,
            ),
        }
    }
}

pub fn spawn_mobs(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut ev_spawn: MessageReader<SpawnMobsOnChunk>,
) {
    let mut rng = rand::rng();

    for SpawnMobsOnChunk(chunk_pos) in ev_spawn.read() {
        let Some(pos) = TilemapData::find_from_center_chunk_size(
            TilemapData::local_pos_to_global(
                *chunk_pos,
                IVec2::new(
                    rng.random_range(0..CHUNK_SIZE as i32),
                    rng.random_range(0..CHUNK_SIZE as i32),
                ),
            ),
            |pos| {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let neigh_pos = pos + IVec2::new(dx, dy);

                        let Some(tile) = tilemap_data.get(neigh_pos) else {
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
            warn!("No valid spawn position found for mobs");
            return;
        };

        let nb_sheeps = rng.random_range(1..=7);
        let nb_boars = rng.random_range(1..=5);

        for _ in 0..nb_sheeps {
            commands.spawn(MobBundle::new(MobId::Sheep, pos));
        }

        for _ in 0..nb_boars {
            commands.spawn(MobBundle::new(MobId::Boar, pos));
        }
    }
}

pub fn update_mobs(tilemap_data: Res<TilemapData>, mut q_mobs: Query<(&mut Mob, &Transform)>) {
    let mut rng = rand::rng();
    for (mut mob, transform) in &mut q_mobs {
        // If already moving, continue
        if !mob.move_queue.is_empty() {
            continue;
        }

        // Wander around
        if rng.random_bool(0.2) {
            let pos = transform_to_pos(transform);
            let directions = tilemap_data.non_blocking_neighbours_pos(pos, true);

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

            let speed = mob.id.data().speed;

            if direction.length() < speed * time.delta_secs() {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                mob.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * speed * time.delta_secs();
                transform.translation.y += dir.y * speed * time.delta_secs();

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}

pub fn update_hostile_mobs(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut q_mobs: Query<(&mut Mob, &Transform)>,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
) {
    let mut rng = rand::rng();
    for (mut mob, transform) in &mut q_mobs {
        // Hostile mobs seek closest dweller within detection radius
        let mob_data = mob.id.data();
        if mob_data.is_hostile() {
            let pos = transform_to_pos(transform);

            let target_dweller = q_dwellers
                .iter_mut()
                .filter_map(|(entity_dweller, dweller, dweller_transform)| {
                    let dweller_pos = transform_to_pos(dweller_transform);
                    let tile_distance_squared = (dweller_pos - pos).length_squared();

                    if tile_distance_squared <= HOSTILE_MOBS_DETECTION_TILE_RADIUS.pow(2) {
                        Some((entity_dweller, dweller, dweller_pos, tile_distance_squared))
                    } else {
                        None
                    }
                })
                .min_by_key(|(_, _, _, tile_distance_squared)| *tile_distance_squared);

            if let Some((entity_dweller, mut dweller, dweller_pos, tile_distance_squared)) =
                target_dweller
            {
                // Attack if close enough
                if tile_distance_squared <= 1 {
                    dweller.health(-(mob_data.attack as i32));
                    commands.entity(entity_dweller).insert(TakingDamage::new());
                } else {
                    // Else pathfind towards dweller
                    let &target_pos = tilemap_data
                        .non_blocking_neighbours_pos(dweller_pos, false)
                        .choose(&mut rng)
                        .unwrap_or(&dweller_pos);

                    mob.pathfind(target_pos, &tilemap_data);
                }
            }
        }
    }
}
