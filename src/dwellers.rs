use bevy::{prelude::*, sprite::Anchor};
use rand::Rng;

use crate::{
    extract_ok,
    tasks::{BuildResult, Task, TaskCompletionEvent, TaskKind, TaskNeeds},
    terrain::{find_from_center, TilemapData, TERRAIN_SIZE, TILE_SIZE},
    tiles::ObjectData,
};

const SPEED: f32 = 80.0;
const Z_INDEX: f32 = 10.0;

#[derive(Component)]
pub struct Dweller {
    pub name: String,
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
                let Some(tile_data) = tilemap_data.0.get(index + IVec2::new(dx, dy)) else {
                    return false;
                };

                if tile_data.is_blocking() {
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
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    mut q_tilemap: Query<&TilemapData>,
    mut q_tasks: Query<(Entity, &mut Task)>,
    mut ev_task_completion: EventWriter<TaskCompletionEvent>,
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
            .iter_mut()
            .find(|(_, task)| task.dweller == Some(entity));

        if let Some((entity_task, mut task)) = task {
            if task.reachable_positions.iter().any(|pos| *pos == index) {
                // Reached task location
                ev_task_completion.send(TaskCompletionEvent { task: entity_task });
            } else {
                // Task moved, try to pathfind again
                let path = task.pathfind(index, tilemap_data);

                if let Some(path) = path {
                    println!("Dweller {} can re-pathfind to {:?}", dweller.name, task);
                    dweller.move_queue = path.0;
                } else {
                    println!("Dweller {} give up {:?}", dweller.name, task);
                    task.dweller = None;
                }
            }

            continue;
        }

        // Get a new task
        // FIXME: dwellers first in the loop can "steal" a task far away from them from a dweller that is closer
        let task_path = q_tasks
            .iter_mut()
            .filter_map(|(_, task)| {
                if task.dweller.is_none() && !task.reachable_positions.is_empty() {
                    match task.needs {
                        TaskNeeds::Nothing => {}
                        TaskNeeds::EmptyHands => {
                            if dweller.object.is_some() {
                                return None;
                            }
                        }
                        TaskNeeds::Object(object_data) => match dweller.object {
                            None => return None,
                            Some(dweller_object) => {
                                if dweller_object != object_data
                                    && !matches!(
                                        task.kind,
                                        TaskKind::Build {
                                            result: BuildResult::Object(build_object),
                                            ..
                                        } if build_object == dweller_object
                                    )
                                {
                                    return None;
                                }
                            }
                        },
                        TaskNeeds::AnyObject => {
                            dweller.object?;
                        }
                        TaskNeeds::Impossible => {
                            return None;
                        }
                    }

                    // Try pathfinding to task
                    let path = task.pathfind(index, tilemap_data);

                    if let Some(path) = path {
                        // println!("Dweller {} can pathfind to {:?}", dweller.name, task);
                        return Some((task, path));
                    }
                }

                None
            })
            .max_by(|(task1, (_, path1)), (task2, (_, path2))| {
                task1
                    .priority
                    .cmp(&task2.priority)
                    .then_with(|| path2.cmp(path1))
            });

        if let Some((mut task, (path, _))) = task_path {
            println!("Dweller {} got task {task:?}", dweller.name);

            task.dweller = Some(entity);
            dweller.move_queue = path;

            continue;
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
