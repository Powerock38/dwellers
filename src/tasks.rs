use bevy::{prelude::*, sprite::Anchor};
use bevy_entitiles::tilemap::map::TilemapStorage;

use crate::{
    dwellers::Dweller,
    extract_ok,
    terrain::{TilemapData, TILE_SIZE},
    tiles::{ObjectData, TileData, TileKind},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TaskKind {
    Dig,
    Smoothen,
    Chop,
    Bridge,
    Pickup,
}

#[derive(Bundle)]
pub struct TaskBundle {
    pub task: Task,
    pub sprite: SpriteBundle,
}

impl TaskBundle {
    pub fn new(task: Task, texture: Handle<Image>) -> Self {
        let x = task.pos.x as f32 * TILE_SIZE;
        let y = task.pos.y as f32 * TILE_SIZE;

        Self {
            task,
            sprite: SpriteBundle {
                texture,
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 1.),
                ..default()
            },
        }
    }
}

#[derive(Debug)]
pub enum TaskNeeds {
    Nothing,
    EmptyHands,
    Object(ObjectData),
}

#[derive(Component, Debug)]
pub struct Task {
    pub kind: TaskKind,
    pub pos: IVec2,
    pub reachable_positions: Vec<IVec2>,
    pub dweller: Option<Entity>,
    pub needs: TaskNeeds,
    pub priority: i32,
}

impl Task {
    pub fn new(pos: IVec2, kind: TaskKind, needs: TaskNeeds, tilemap_data: &TilemapData) -> Self {
        Self {
            kind,
            pos,
            reachable_positions: Self::compute_reachable_positions(pos, tilemap_data),
            dweller: None,
            priority: 0,
            needs,
        }
    }

    pub fn priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    pub fn recompute_reachable_positions(&mut self, tilemap_data: &TilemapData) {
        self.reachable_positions = Self::compute_reachable_positions(self.pos, tilemap_data);
    }

    fn compute_reachable_positions(pos: IVec2, tilemap_data: &TilemapData) -> Vec<IVec2> {
        if let Some(tile_data) = tilemap_data.0.get(pos) {
            if !tile_data.is_blocking() {
                return vec![pos];
            }
        }

        tilemap_data.non_blocking_neighbours_pos(pos)
    }
}

pub fn update_unreachable_tasks(
    q_tilemap: Query<&TilemapData, Changed<TilemapData>>,
    mut q_tasks: Query<&mut Task>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    for mut task in &mut q_tasks {
        if task.reachable_positions.is_empty() || task.dweller.is_none() {
            task.recompute_reachable_positions(tilemap_data);
        }
    }
}

#[derive(Event)]
pub struct TaskCompletionEvent {
    pub task: Entity,
}

pub fn event_task_completion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<TaskCompletionEvent>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
    mut q_dwellers: Query<&mut Dweller>,
    mut q_tasks: Query<(Entity, &mut Task)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    let mut update_tasks = false;

    for event in events.read() {
        let Ok((entity, task)) = q_tasks.get(event.task) else {
            continue;
        };

        let Ok(mut dweller) = q_dwellers.get_mut(task.dweller.unwrap()) else {
            continue;
        };

        let Some(tile_data) = tilemap_data.0.get(task.pos) else {
            continue;
        };

        let mut success = false;

        match task.kind {
            TaskKind::Dig => {
                if tile_data.is_blocking() {
                    TileData::STONE_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Dug tile at {:?}", task.pos);
                    update_tasks = true;
                    success = true;
                }
            }

            TaskKind::Smoothen => {
                if tile_data == TileData::DIRT_WALL || tile_data == TileData::STONE_WALL {
                    TileData::DUNGEON_WALL.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Smoothened wall at {:?}", task.pos);
                    success = true;
                } else if tile_data == TileData::STONE_FLOOR {
                    TileData::DUNGEON_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Smoothened floor at {:?}", task.pos);
                    success = true;
                }
            }

            TaskKind::Chop => {
                if tile_data == TileData::TREE {
                    TileData::GRASS_FLOOR.with(ObjectData::WOOD).set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    commands.spawn(TaskBundle::new(
                        Task::new(
                            task.pos,
                            TaskKind::Pickup,
                            TaskNeeds::EmptyHands,
                            &tilemap_data,
                        ),
                        asset_server.load("sprites/pickup.png"),
                    ));

                    println!("Chopped tile at {:?}", task.pos);
                    update_tasks = true;
                    success = true;
                }
            }

            TaskKind::Bridge => {
                if tile_data == TileData::WATER {
                    TileData::BRIDGE_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Bridged tile at {:?}", task.pos);
                    update_tasks = true;
                    success = true;
                }
            }

            TaskKind::Pickup => {
                if let TileKind::Floor(Some(object_data)) = tile_data.kind {
                    if dweller.object.is_none() {
                        let mut new_tile_data = tile_data;
                        new_tile_data.kind = TileKind::Floor(None);

                        new_tile_data.set_at(
                            task.pos,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        dweller.object = Some(object_data);

                        println!("Picked up object at {:?}", task.pos);
                        success = true;
                    }
                }
            }
        }

        if success {
            if let TaskNeeds::Object(object_data) = task.needs {
                if let Some(dweller_object) = dweller.object {
                    if dweller_object == object_data {
                        dweller.object = None;
                    }
                } else {
                    println!("SHOULD NEVER HAPPEN: Dweller {} completed task {:?} without needed object {:?}", dweller.name, task, object_data);
                }
            }

            commands.entity(entity).despawn();
        } else {
            println!(
                "SHOULD NEVER HAPPEN: Dweller {} failed task {:?}",
                dweller.name, task
            );
        }
    }

    if update_tasks {
        for (_, mut o_task) in &mut q_tasks {
            o_task.recompute_reachable_positions(&tilemap_data);
        }
    }
}
