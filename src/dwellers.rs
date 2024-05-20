use bevy::prelude::*;
use bevy_entitiles::{
    algorithm::pathfinding::{PathFinder, PathFindingQueue},
    math::extension::TileIndex,
    tilemap::map::TilemapType,
};
use rand::Rng;

use crate::{
    extract_ok,
    tasks::Task,
    terrain::{MineTile, TilemapData, TILE_SIZE},
};

#[derive(Component)]
pub struct Dweller {
    name: String,
    age: u32,
    health: u32,
    pub next_move: Option<IVec2>,
}

#[derive(Bundle)]
pub struct DwellerBundle {
    dweller: Dweller,
    sprite: SpriteBundle,
}

pub fn spawn_dwellers(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DwellerBundle {
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/dweller.png"),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::TopLeft,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        },
        dweller: Dweller {
            name: "Alice".to_string(),
            age: 30,
            health: 100,
            next_move: None,
        },
    });
}

/*
if task in Children
    Schedule pathfinding to task
else
    Search for available tasks (no Parent component)
    Reparent task to dweller
if still no task
    Wander around
*/

pub fn update_dwellers(
    mut commands: Commands,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform, Option<&Children>)>,
    mut q_tilemap: Query<(&TilemapData, &mut PathFindingQueue)>,
    q_tasks_available: Query<(Entity, &Task), Without<Parent>>,
    q_tasks_dwellers: Query<&Task, With<Parent>>,
    mut ev_mine_tile: EventWriter<MineTile>,
) {
    let (tilemap_data, mut pathfinding_queue) = extract_ok!(q_tilemap.get_single_mut());

    for (entity, mut dweller, transform, children) in q_dwellers.iter_mut() {
        let mut index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        let mut has_task = false;

        if let Some(children) = children {
            for entity_task in children {
                let Ok(task) = q_tasks_dwellers.get(*entity_task) else {
                    continue;
                };

                match task {
                    Task::Dig(pos) => {
                        if pos
                            .neighbours(TilemapType::Square, false)
                            .iter()
                            .any(|pos| pos.map_or(false, |pos| pos == index))
                        {
                            // Arrived at task location
                            ev_mine_tile.send(MineTile(*pos));
                            commands.entity(*entity_task).despawn();
                        } else {
                            pathfinding_queue.schedule(
                                *entity_task,
                                PathFinder {
                                    origin: index,
                                    dest: *pos,
                                    allow_diagonal: false,
                                    max_steps: None,
                                },
                            );
                        }
                    }
                }

                has_task = true;
                break;
            }
        }

        if !has_task {
            for (entity_task, task) in &q_tasks_available {
                commands.entity(entity_task).set_parent(entity);
                has_task = true;
                break;
            }
        }

        if !has_task {
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

            if !tiledata.wall {
                dweller.next_move = Some(index);
            }
        }
    }
}

pub fn update_dwellers_next_move(mut q_dwellers: Query<(&mut Dweller, &mut Transform)>) {
    for (mut dweller, mut transform) in &mut q_dwellers {
        if let Some(next_move) = dweller.next_move {
            transform.translation.x = next_move.x as f32 * TILE_SIZE;
            transform.translation.y = next_move.y as f32 * TILE_SIZE;

            dweller.next_move = None;
        }
    }
}
