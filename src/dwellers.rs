use bevy::{prelude::*, sprite::Anchor};
use bevy_entitiles::{
    algorithm::pathfinding::{PathFinder, PathFindingQueue},
    math::extension::TileIndex,
    prelude::Path,
    tilemap::map::TilemapType,
};
use rand::{seq::IteratorRandom, Rng};

use crate::{
    extract_ok,
    tasks::{Task, TaskKind},
    terrain::{TilemapData, TILE_SIZE},
    tiles::{MineTile, SmoothenTile},
};

#[derive(Component)]
pub struct Dweller {
    name: String,
    speed: f32,
    move_queue: Vec<IVec2>, // next move is at the end
}

#[derive(Bundle)]
pub struct DwellerBundle {
    dweller: Dweller,
    sprite: SpriteBundle,
}

pub fn spawn_dwellers(mut commands: Commands, asset_server: Res<AssetServer>) {
    for name in ["Alice", "Bob", "Charlie", "Dave", "Eve"] {
        commands.spawn(DwellerBundle {
            sprite: SpriteBundle {
                texture: asset_server.load("sprites/dweller.png"),
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 10.0),
                ..default()
            },
            dweller: Dweller {
                name: name.to_string(),
                speed: 1.0,
                move_queue: vec![],
            },
        });
    }
}

pub fn update_dwellers(
    mut commands: Commands,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    mut q_tilemap: Query<(&TilemapData, &mut PathFindingQueue)>,
    mut q_tasks: Query<(Entity, &mut Task)>,
    mut ev_mine_tile: EventWriter<MineTile>,
    mut ev_smoothen_tile: EventWriter<SmoothenTile>,
) {
    let (tilemap_data, mut pathfinding_queue) = extract_ok!(q_tilemap.get_single_mut());

    for (entity, mut dweller, transform) in &mut q_dwellers {
        if !dweller.move_queue.is_empty() {
            continue;
        }

        let mut index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        let mut has_task = false;

        // Check if dweller has a task assigned in all tasks
        for (entity_task, task) in &mut q_tasks {
            if Some(entity) == task.dweller {
                if task
                    .pos
                    .neighbours(TilemapType::Square, false)
                    .iter()
                    .any(|pos| pos.map_or(false, |pos| pos == index))
                {
                    // Reached task location
                    match task.kind {
                        TaskKind::Dig => {
                            ev_mine_tile.send(MineTile(task.pos));
                            println!("Dweller {} mining at {:?}", dweller.name, task.pos);
                        }
                        TaskKind::Smoothen => {
                            ev_smoothen_tile.send(SmoothenTile(task.pos));
                            println!("Dweller {} smoothening at {:?}", dweller.name, task.pos);
                        }
                    }

                    commands.entity(entity_task).despawn();
                } else {
                    // Pathfind to random non-wall adjacent tile
                    let mut rng = rand::thread_rng();

                    let random_dest = task.pos_adjacent.iter().choose(&mut rng);

                    if let Some(&dest) = random_dest {
                        pathfinding_queue.schedule(
                            entity_task,
                            PathFinder {
                                origin: index,
                                dest,
                                allow_diagonal: false,
                                max_steps: None,
                            },
                        );

                        println!("Dweller {} pathfinding to {:?}", dweller.name, task.pos);
                    } else {
                        println!(
                            "SHOULD NEVER HAPPEN: Dweller {} selected unreachable {task:?}",
                            dweller.name
                        );
                    }
                }

                has_task = true;
                break;
            }
        }

        // Get a new task
        if !has_task {
            for (_, mut task) in &mut q_tasks {
                if task.dweller.is_some() {
                    continue;
                }

                if task.pos_adjacent.is_empty() {
                    continue;
                }

                task.dweller = Some(entity);
                has_task = true;

                println!("Dweller {} got task {task:?}", dweller.name);

                break;
            }
        }

        // Wander around
        if !has_task {
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
}

pub fn update_pathfinding_tasks(
    mut commands: Commands,
    q_tasks: Query<(Entity, &Task, &Path)>,
    mut q_dwellers: Query<&mut Dweller>,
) {
    for (entity_task, task, path) in &q_tasks {
        if let Some(entity_dweller) = task.dweller {
            let Ok(mut dweller) = q_dwellers.get_mut(entity_dweller) else {
                continue;
            };

            dweller.move_queue = path.iter().copied().collect();

            commands.entity(entity_task).remove::<Path>();

            println!("Dweller {} got path {:?}", dweller.name, dweller.move_queue);
        }
    }
}

pub fn update_dwellers_movement(
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

            if direction.length() < dweller.speed {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                dweller.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * dweller.speed;
                transform.translation.y += dir.y * dweller.speed;

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}
