use bevy::{prelude::*, sprite::Anchor};
use pathfinding::prelude::astar;
use rand::Rng;

use crate::{
    extract_ok,
    tasks::{Task, TaskKind},
    terrain::{find_from_center, TilemapData, TERRAIN_SIZE, TILE_SIZE},
    tiles::{ObjectData, SetTileEvent, TileData, TileEvent},
    utils::manhattan_distance,
};

const SPEED: f32 = 80.0;
const Z_INDEX: f32 = 10.0;

#[derive(Component)]
pub struct Dweller {
    name: String,
    speed: f32,
    move_queue: Vec<IVec2>, // next move is at the end
    pub object: Option<ObjectData>,
}

#[derive(Bundle)]
pub struct DwellerBundle {
    dweller: Dweller,
    sprite: SpriteBundle,
}

pub fn spawn_dwellers(
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
        println!("No valid spawn position found for dwellers");
        return;
    };

    println!("Dwellers spawn position: {spawn_pos:?}");

    for name in ["Alice", "Bob", "Charlie", "Dave", "Eve"] {
        commands.spawn(DwellerBundle {
            sprite: SpriteBundle {
                texture: asset_server.load("sprites/dweller.png"),
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform::from_xyz(
                    spawn_pos.x as f32 * TILE_SIZE,
                    spawn_pos.y as f32 * TILE_SIZE,
                    Z_INDEX,
                ),
                ..default()
            },
            dweller: Dweller {
                name: name.to_string(),
                speed: SPEED,
                move_queue: vec![],
                object: None,
            },
        });
    }
}
pub fn update_dwellers(
    mut commands: Commands,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    mut q_tilemap: Query<&TilemapData>,
    mut q_tasks: Query<(Entity, &mut Task)>,
    mut ev_set_tile: EventWriter<SetTileEvent>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single_mut());

    for (entity, mut dweller, transform) in &mut q_dwellers {
        if !dweller.move_queue.is_empty() {
            continue;
        }

        let mut index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        // Check if dweller has a task assigned in all tasks
        let task = q_tasks
            .iter()
            .filter(|(_, task)| task.dweller == Some(entity))
            .max_by_key(|(_, task)| task.priority);

        if let Some((entity_task, task)) = task {
            if task.reachable_positions.iter().any(|pos| *pos == index) {
                // Reached task location
                match task.kind {
                    TaskKind::Dig => {
                        ev_set_tile.send(SetTileEvent::new(task.pos, TileEvent::Dig));
                    }

                    TaskKind::Smoothen => {
                        ev_set_tile.send(SetTileEvent::new(task.pos, TileEvent::Smoothen));
                    }

                    TaskKind::Chop => {
                        ev_set_tile.send(SetTileEvent::new(task.pos, TileEvent::Chop));
                    }

                    TaskKind::Bridge => {
                        ev_set_tile.send(SetTileEvent::new(task.pos, TileEvent::Bridge));
                    }

                    TaskKind::Pickup => {
                        if dweller.object.is_none() {
                            ev_set_tile
                                .send(SetTileEvent::new(task.pos, TileEvent::Pickup(entity)));
                        }
                    }
                }

                commands.entity(entity_task).despawn();
            }

            return;
        }

        // Get a new task
        let task_path = q_tasks
            .iter_mut()
            .filter_map(|(_, task)| {
                if task.dweller.is_none() && !task.reachable_positions.is_empty() {
                    if let Some(object_data) = task.needs_object {
                        if dweller.object != Some(object_data) {
                            // TODO
                            // Search closest pathfindable object_data (find_from_center)
                            // "rebrand" task as Pickup @ object_data with higher priority
                            // add original task to queue
                            // PROBLEM: dweller needs to accept og task before they're sure it's the task to choose
                        }
                    }

                    // Try pathfinding to task

                    let path = task
                        .reachable_positions
                        .iter()
                        .filter_map(|pos| {
                            astar(
                                pos,
                                |p| {
                                    tilemap_data
                                        .non_blocking_neighbours(*p)
                                        .into_iter()
                                        .map(|p| (p, 1))
                                },
                                |p| manhattan_distance(*p, index),
                                |p| *p == index,
                            )
                        })
                        .min_by_key(|path| path.1);

                    if let Some(path) = path {
                        println!("Dweller can {} pathfind to {:?}", dweller.name, task);
                        return Some((task, path));
                    }
                }

                None
            })
            .min_by_key(|(_, path)| path.1);

        if let Some((mut task, (path, _))) = task_path {
            println!("Dweller {} got task {task:?}", dweller.name);

            task.dweller = Some(entity);
            dweller.move_queue = path;

            return;
        }

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
            dweller.move_queue.push(index);
        }
    }
}

pub fn update_dwellers_movement(
    time: Res<Time>,
    mut q_dwellers: Query<(&mut Dweller, &mut Transform, &mut Sprite)>,
) {
    for (mut dweller, mut transform, mut sprite) in &mut q_dwellers {
        // Move to next position in queue

        if let Some(next_move) = dweller.move_queue.last() {
            let target = Vec2::new(
                next_move.x as f32 * TILE_SIZE,
                next_move.y as f32 * TILE_SIZE,
            );

            let direction = target - transform.translation.truncate();

            if direction.length() < dweller.speed * time.delta_seconds() {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                dweller.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * dweller.speed * time.delta_seconds();
                transform.translation.y += dir.y * dweller.speed * time.delta_seconds();

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}
