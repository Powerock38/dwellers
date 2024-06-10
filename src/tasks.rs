use bevy::{prelude::*, sprite::Anchor};
use bevy_entitiles::tilemap::map::TilemapStorage;
use pathfinding::directed::astar::astar;

use crate::{
    dwellers::Dweller,
    extract_ok,
    mobs::Mob,
    terrain::{find_from_center, TilemapData, TILE_SIZE},
    tiles::{ObjectData, TileData, TileKind},
    utils::manhattan_distance,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TaskKind {
    Dig,
    Smoothen,
    Chop,
    Bridge,
    Pickup,
    BuildObject {
        object: ObjectData,
        cost: ObjectData,
    },
    Hunt,
    Stockpile,
}

impl TaskKind {
    pub fn can_be_completed(&self, tile_data: TileData) -> bool {
        match self {
            TaskKind::Dig => {
                tile_data == TileData::DIRT_WALL
                    || tile_data == TileData::STONE_WALL
                    || tile_data == TileData::DUNGEON_WALL
            }
            TaskKind::Smoothen => {
                tile_data == TileData::DIRT_WALL
                    || tile_data == TileData::STONE_WALL
                    || tile_data == TileData::STONE_FLOOR
            }
            TaskKind::Chop => tile_data.kind == TileKind::Floor(Some(ObjectData::TREE)),
            TaskKind::Bridge => tile_data == TileData::WATER,
            TaskKind::BuildObject { .. } => tile_data.kind == TileKind::Floor(None),
            TaskKind::Pickup => {
                matches!(tile_data.kind, TileKind::Floor(Some(object)) if object.carriable())
            }
            TaskKind::Hunt => true,
            TaskKind::Stockpile => matches!(tile_data.kind, TileKind::Floor(_)),
        }
    }
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
    AnyObject,
    Impossible,
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

    pub fn pathfind(
        &self,
        dweller_pos: IVec2,
        tilemap_data: &TilemapData,
    ) -> Option<(Vec<IVec2>, i32)> {
        self.reachable_positions
            .iter()
            .filter_map(|pos| {
                astar(
                    pos,
                    |p| {
                        tilemap_data
                            .non_blocking_neighbours_pos(*p)
                            .into_iter()
                            .map(|p| (p, 1))
                    },
                    |p| manhattan_distance(*p, dweller_pos),
                    |p| *p == dweller_pos,
                )
            })
            .min_by_key(|path| path.1)
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
    q_mobs: Query<(Entity, &Mob, &Transform)>,
    mut q_dwellers: Query<(&mut Dweller, &Transform)>,
    mut q_tasks: Query<(Entity, &mut Task, Option<&Parent>)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    let mut update_tasks_pos = false;

    for event in events.read() {
        let Ok((entity, mut task, task_parent)) = q_tasks.get_mut(event.task) else {
            continue;
        };

        let Some((mut dweller, dweller_transform)) =
            task.dweller.and_then(|d| q_dwellers.get_mut(d).ok())
        else {
            continue;
        };

        let Some(tile_data) = tilemap_data.0.get(task.pos) else {
            continue;
        };

        let mut success = false;

        if task.kind.can_be_completed(tile_data) {
            match task.kind {
                TaskKind::Dig => {
                    TileData::STONE_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Dug tile at {:?}", task.pos);
                    update_tasks_pos = true;
                    success = true;
                }

                TaskKind::Smoothen => {
                    let tile = match tile_data.kind {
                        TileKind::Wall => TileData::DUNGEON_WALL,
                        TileKind::Floor(None) => TileData::DUNGEON_FLOOR,
                        TileKind::Floor(Some(object)) => TileData::DUNGEON_FLOOR.with(object),
                    };

                    tile.set_at(task.pos, &mut commands, &mut tilemap, &mut tilemap_data);

                    println!("Smoothened tile at {:?}", task.pos);
                    success = true;
                }

                TaskKind::Chop => {
                    tile_data.with(ObjectData::WOOD).set_at(
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
                    update_tasks_pos = true;
                    success = true;
                }

                TaskKind::Bridge => {
                    TileData::BRIDGE_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Bridged tile at {:?}", task.pos);
                    update_tasks_pos = true;
                    success = true;
                }

                TaskKind::Pickup => {
                    if let TileKind::Floor(Some(object_data)) = tile_data.kind {
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

                TaskKind::BuildObject { object, .. } => {
                    tile_data.with(object).set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    println!("Built {:?} at {:?}", object, task.pos);
                    update_tasks_pos = true;
                    success = true;
                }

                TaskKind::Hunt => {
                    if let Some(task_parent) = task_parent.map(Parent::get) {
                        if let Ok((entity_mob, mob, mob_transform)) = q_mobs.get(task_parent) {
                            let mob_pos = (mob_transform.translation / TILE_SIZE)
                                .truncate()
                                .as_ivec2();

                            if dweller_transform
                                .translation
                                .distance(mob_transform.translation)
                                < TILE_SIZE
                            {
                                if let Some(loot_tile_data) = tilemap_data.0.get(mob_pos) {
                                    if loot_tile_data.kind == TileKind::Floor(None) {
                                        loot_tile_data.with(mob.loot).set_at(
                                            mob_pos,
                                            &mut commands,
                                            &mut tilemap,
                                            &mut tilemap_data,
                                        );

                                        commands.spawn(TaskBundle::new(
                                            Task::new(
                                                mob_pos,
                                                TaskKind::Pickup,
                                                TaskNeeds::EmptyHands,
                                                &tilemap_data,
                                            ),
                                            asset_server.load("sprites/pickup.png"),
                                        ));
                                    }
                                }

                                commands.entity(entity_mob).despawn_recursive();

                                println!("Hunted mob at {:?}", mob_transform.translation);
                                success = true;
                            } else {
                                task.pos = mob_pos;
                                task.recompute_reachable_positions(&tilemap_data);
                            }
                        }
                    }
                }

                TaskKind::Stockpile => {
                    if tile_data.kind == TileKind::Floor(None) {
                        if let Some(object) = dweller.object {
                            tile_data.with(object).set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );

                            println!("Stockpiled object at {:?}", task.pos);
                            update_tasks_pos = true;
                            success = true;
                        }
                    }
                }
            }
        }

        if success {
            match task.needs {
                TaskNeeds::Object(object_data) => {
                    if let Some(dweller_object) = dweller.object {
                        if dweller_object == object_data
                            || matches!(
                                task.kind,
                                TaskKind::BuildObject {
                                    object: build_object,
                                    ..
                                } if build_object == dweller_object
                            )
                        {
                            dweller.object = None;
                        }
                    } else {
                        println!("SHOULD NEVER HAPPEN: Dweller {} completed task {:?} without needed object {:?}", dweller.name, task, object_data);
                    }
                }

                TaskNeeds::AnyObject => {
                    if let Some(dweller_object) = dweller.object {
                        if dweller_object != ObjectData::TREE {
                            dweller.object = None;
                        }
                    } else {
                        println!("SHOULD NEVER HAPPEN: Dweller {} completed task {:?} without any object", dweller.name, task);
                    }
                }

                TaskNeeds::Impossible => {
                    println!(
                        "SHOULD NEVER HAPPEN: Dweller {} completed impossible task {:?}",
                        dweller.name, task
                    );
                }

                TaskNeeds::EmptyHands => {
                    if dweller.object.is_some() {
                        println!(
                            "SHOULD NEVER HAPPEN: Dweller {} completed task {:?} with object (needed empty hands)",
                            dweller.name, task
                        );
                    }
                }

                TaskNeeds::Nothing => {}
            }

            if matches!(task.kind, TaskKind::Stockpile) {
                task.dweller = None;
                task.needs = TaskNeeds::Impossible;
            } else {
                commands.entity(entity).despawn();
            }
        } else {
            println!("Dweller {} failed task {:?}", dweller.name, task);
        }
    }

    if update_tasks_pos {
        for (_, mut o_task, _) in &mut q_tasks {
            o_task.recompute_reachable_positions(&tilemap_data);
        }
    }
}

pub fn update_pickups(
    mut commands: Commands,
    q_tilemap: Query<&TilemapData>,
    asset_server: Res<AssetServer>,
    q_new_tasks: Query<&Task, Added<Task>>,
    q_tasks: Query<&Task>,
    q_dwellers: Query<&Dweller>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    for task in &q_new_tasks {
        if matches!(task.needs, TaskNeeds::AnyObject | TaskNeeds::Object(_)) {
            let specific_object = match task.needs {
                TaskNeeds::Object(object) => Some(object),
                TaskNeeds::AnyObject => None,
                _ => continue, // unreachable
            };

            // check if it needs a new Pickup task: check for already existing Pickup tasks, and Dwellers with the required object
            if q_tasks.iter().any(|t| t.kind == TaskKind::Pickup)
                || q_dwellers.iter().any(|dweller| {
                    dweller.object.is_some()
                        && (specific_object.is_none() || specific_object == dweller.object)
                })
            {
                continue;
            }

            // Find object: search around task.pos
            let index = find_from_center(task.pos, |index| {
                if let Some(tile_data) = tilemap_data.0.get(index) {
                    if let TileKind::Floor(Some(object)) = tile_data.kind {
                        if specific_object.is_none() || specific_object == Some(object) {
                            return TaskKind::Pickup.can_be_completed(tile_data);
                        }
                    }
                }

                false
            });

            if let Some(index) = index {
                commands.spawn(TaskBundle::new(
                    Task::new(index, TaskKind::Pickup, TaskNeeds::EmptyHands, tilemap_data),
                    asset_server.load("sprites/pickup.png"),
                ));
            }
        }
    }
}
