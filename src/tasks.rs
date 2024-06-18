use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};
use bevy_entitiles::tilemap::map::TilemapStorage;
use pathfinding::directed::astar::astar;
use rand::Rng;

use crate::{
    dwellers::Dweller,
    extract_ok,
    mobs::Mob,
    tilemap::{TilemapData, TILE_SIZE},
    tiles::{ObjectData, TileData, TileKind},
    utils::manhattan_distance,
    SpriteLoaderBundle,
};

#[derive(PartialEq, Clone, Copy, Reflect, Default, Debug)]
pub enum TaskKind {
    #[default]
    Dig,
    Smoothen,
    Harvest,
    Bridge,
    Pickup,
    Hunt,
    Stockpile,
    Build {
        result: BuildResult,
        cost: ObjectData,
    },
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
            TaskKind::Harvest => {
                matches!(
                    tile_data.kind,
                    TileKind::Floor(Some(
                        ObjectData::TREE | ObjectData::TALL_GRASS | ObjectData::WHEAT_PLANT
                    ))
                )
            }
            TaskKind::Bridge => tile_data == TileData::WATER,
            TaskKind::Build { .. } => tile_data.kind == TileKind::Floor(None),
            TaskKind::Pickup => {
                matches!(tile_data.kind, TileKind::Floor(Some(object)) if object.carriable())
            }
            TaskKind::Hunt => true,
            TaskKind::Stockpile => {
                matches!(tile_data.kind, TileKind::Floor(object) if object.map_or(true, ObjectData::carriable))
            }
        }
    }

    pub fn id(&self) -> String {
        format!("{self:?}")
            .to_lowercase()
            .split_whitespace()
            .next()
            .unwrap()
            .to_string()
    }
}

#[derive(PartialEq, Clone, Copy, Reflect, Debug)]
pub enum BuildResult {
    Object(ObjectData),
    Tile(TileData),
}

// dumb impl only to make Reflect happy
impl Default for BuildResult {
    fn default() -> Self {
        Self::Object(ObjectData::default())
    }
}

#[derive(Bundle)]
pub struct TaskBundle {
    pub task: Task,
    pub sprite: SpriteLoaderBundle,
}

impl TaskBundle {
    pub fn new(task: Task) -> Self {
        let x = task.pos.x as f32 * TILE_SIZE;
        let y = task.pos.y as f32 * TILE_SIZE;

        let texture_path = format!("sprites/{}.png", task.kind.id());

        Self {
            task,
            sprite: SpriteLoaderBundle::new(texture_path.as_str(), x, y, 1.),
        }
    }
}

#[derive(PartialEq, Reflect, Default, Debug)]
pub enum TaskNeeds {
    #[default]
    Nothing,
    EmptyHands,
    Object(ObjectData),
    AnyObject,
    Impossible,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component, MapEntities)]
pub struct Task {
    pub kind: TaskKind,
    pub pos: IVec2,
    pub reachable_positions: Vec<IVec2>,
    pub dweller: Option<Entity>,
    pub needs: TaskNeeds,
    pub priority: i32,
}

impl MapEntities for Task {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.dweller {
            self.dweller = Some(entity_mapper.map_entity(entity));
        }
    }
}

impl Task {
    pub fn new(pos: IVec2, kind: TaskKind, needs: TaskNeeds, tilemap_data: &TilemapData) -> Self {
        let mut task = Self {
            kind,
            pos,
            reachable_positions: vec![],
            dweller: None,
            priority: 0,
            needs,
        };
        task.recompute_reachable_positions(tilemap_data);
        task
    }

    pub fn priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    pub fn recompute_reachable_positions(&mut self, tilemap_data: &TilemapData) {
        self.reachable_positions = self.compute_reachable_positions(self.pos, tilemap_data);
    }

    fn compute_reachable_positions(&self, pos: IVec2, tilemap_data: &TilemapData) -> Vec<IVec2> {
        if let Some(tile_data) = tilemap_data.get(pos) {
            let will_build_blocking_tile = if let TaskKind::Build {
                result: BuildResult::Tile(tile),
                ..
            } = self.kind
            {
                tile.is_blocking()
            } else {
                false
            };

            if !tile_data.is_blocking() && !will_build_blocking_tile {
                return vec![pos];
            }
        }

        tilemap_data.non_blocking_neighbours_pos(pos, false)
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
                            .non_blocking_neighbours_pos(*p, true)
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
    mut events: EventReader<TaskCompletionEvent>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
    q_mobs: Query<(Entity, &Mob, &Transform)>,
    mut q_dwellers: Query<(&mut Dweller, &Transform)>,
    mut q_tasks: Query<(Entity, &mut Task, Option<&Parent>)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    let mut update_tasks_pos = false;
    let mut update_stockpiles = false;

    for event in events.read() {
        let Ok((entity, mut task, task_parent)) = q_tasks.get_mut(event.task) else {
            continue;
        };

        let Some((mut dweller, dweller_transform)) =
            task.dweller.and_then(|d| q_dwellers.get_mut(d).ok())
        else {
            continue;
        };

        let Some(tile_data) = tilemap_data.get(task.pos) else {
            continue;
        };

        let mut rng = rand::thread_rng();

        let mut success = false;

        if task.kind.can_be_completed(tile_data) {
            match task.kind {
                TaskKind::Dig => {
                    let tile = if rng.gen_bool(0.2) {
                        commands.spawn(TaskBundle::new(Task::new(
                            task.pos,
                            TaskKind::Pickup,
                            TaskNeeds::EmptyHands,
                            &tilemap_data,
                        )));

                        TileData::STONE_FLOOR.with(ObjectData::ROCK)
                    } else {
                        TileData::STONE_FLOOR
                    };

                    tile.set_at(task.pos, &mut commands, &mut tilemap, &mut tilemap_data);

                    debug!("Dug tile at {:?}", task.pos);
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

                    debug!("Smoothened tile at {:?}", task.pos);
                    success = true;
                }

                TaskKind::Harvest => {
                    if let Some(object) = match tile_data.kind {
                        TileKind::Floor(Some(ObjectData::TREE)) => {
                            if rng.gen_bool(0.3) {
                                Some(Some(ObjectData::WOOD))
                            } else {
                                Some(None)
                            }
                        }

                        TileKind::Floor(Some(ObjectData::TALL_GRASS)) => {
                            Some(Some(ObjectData::SEEDS))
                        }

                        TileKind::Floor(Some(ObjectData::WHEAT_PLANT)) => {
                            dweller.object = Some(ObjectData::WHEAT);

                            Some(Some(ObjectData::FARM))
                        }
                        _ => None,
                    } {
                        if let Some(object) = object {
                            tile_data.with(object).set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );

                            if object.carriable() {
                                commands.spawn(TaskBundle::new(Task::new(
                                    task.pos,
                                    TaskKind::Pickup,
                                    TaskNeeds::EmptyHands,
                                    &tilemap_data,
                                )));
                            }
                        } else {
                            tile_data.without_object().set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );
                        }

                        debug!("Harvested object at {:?}", task.pos);
                        update_tasks_pos = true;
                        success = true;
                    }
                }

                TaskKind::Bridge => {
                    TileData::BRIDGE_FLOOR.set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    debug!("Bridged tile at {:?}", task.pos);
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

                        debug!("Picked up object at {:?}", task.pos);
                        update_stockpiles = true;
                        success = true;
                    }
                }

                TaskKind::Build { result, .. } => {
                    match result {
                        BuildResult::Object(object) => {
                            tile_data.with(object).set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );
                        }
                        BuildResult::Tile(tile) => {
                            tile.set_at(task.pos, &mut commands, &mut tilemap, &mut tilemap_data);
                        }
                    }

                    debug!("Built {:?} at {:?}", result, task.pos);
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
                                if let Some(loot_tile_data) = tilemap_data.get(mob_pos) {
                                    if loot_tile_data.kind == TileKind::Floor(None) {
                                        loot_tile_data.with(mob.loot).set_at(
                                            mob_pos,
                                            &mut commands,
                                            &mut tilemap,
                                            &mut tilemap_data,
                                        );

                                        commands.spawn(TaskBundle::new(Task::new(
                                            mob_pos,
                                            TaskKind::Pickup,
                                            TaskNeeds::EmptyHands,
                                            &tilemap_data,
                                        )));
                                    }
                                }

                                commands.entity(entity_mob).despawn_recursive();

                                debug!("Hunted mob at {:?}", mob_transform.translation);
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

                            debug!("Stockpiled object at {:?}", task.pos);
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
                                TaskKind::Build {
                                    result: BuildResult::Object(build_object),
                                    ..
                                } if build_object == dweller_object
                            )
                        {
                            dweller.object = None;
                        }
                    } else {
                        error!("SHOULD NEVER HAPPEN: Dweller {} completed task {:?} without needed object {:?}", dweller.name, task, object_data);
                    }
                }

                TaskNeeds::AnyObject => {
                    if dweller.object.is_some() {
                        dweller.object = None;
                    } else {
                        error!("SHOULD NEVER HAPPEN: Dweller {} completed task {:?} without any object", dweller.name, task);
                    }
                }

                TaskNeeds::Impossible => {
                    error!(
                        "SHOULD NEVER HAPPEN: Dweller {} completed impossible task {:?}",
                        dweller.name, task
                    );
                }

                TaskNeeds::EmptyHands | TaskNeeds::Nothing => {}
            }

            if matches!(task.kind, TaskKind::Stockpile) {
                task.dweller = None;
                task.needs = TaskNeeds::Impossible;
            } else {
                commands.entity(entity).despawn();
            }
        } else {
            info!("Dweller {} failed task {:?}", dweller.name, task);
        }
    }

    if update_tasks_pos {
        for (_, mut task, _) in &mut q_tasks {
            task.recompute_reachable_positions(&tilemap_data);
        }
    }

    if update_stockpiles {
        for (_, mut task, _) in &mut q_tasks {
            if matches!(task.kind, TaskKind::Stockpile)
                && tilemap_data
                    .get(task.pos)
                    .map_or(false, |tile_data| tile_data.kind == TileKind::Floor(None))
            {
                task.needs = TaskNeeds::AnyObject;
            }
        }
    }
}

pub fn update_pickups(
    mut commands: Commands,
    q_tilemap: Query<&TilemapData>,
    q_new_tasks: Query<&Task, Added<Task>>,
    q_tasks: Query<&Task>, // Without<Added<Task>> ?
    q_dwellers: Query<(Entity, &Dweller)>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    let mut task_indexes = vec![];

    for task in &q_new_tasks {
        let (specific_object, needs_object) = match task.needs {
            TaskNeeds::Object(object) => (Some(object), true),
            TaskNeeds::AnyObject => (None, true),
            _ => (None, false),
        };

        if needs_object {
            // check if it needs a new Pickup task: check for already existing Pickup tasks for the required object
            if q_tasks.iter().any(|t| {
                t.kind == TaskKind::Pickup
                && t.dweller.is_none() // not being worked on
                    && tilemap_data.get(t.pos).is_some_and(|tile_data| {
                        if let TileKind::Floor(Some(object_data)) = tile_data.kind {
                            if let Some(object) = specific_object {
                                return object == object_data;
                            }

                            return true;
                        }

                        error!("SHOULD NEVER HAPPEN: Pickup task at {:?} has no object", t.pos);
                        false
                    })
            }) || // or Dwellers with the required object
            q_dwellers.iter().any(|(entity_dweller, dweller)| {
                if let Some(dweller_object) = dweller.object {
                 (specific_object.is_none() || specific_object == dweller.object)
                    && // not working on a task that needs it
                    !q_tasks.iter().any(|t| {
                        t.dweller == Some(entity_dweller) && t.needs == TaskNeeds::Object(dweller_object)
                    })
                } else {
                    false
                }
            }) {
                continue;
            }

            // Find object: search around task.pos
            let index = TilemapData::find_from_center(task.pos, |index| {
                if let Some(tile_data) = tilemap_data.get(index) {
                    if let TileKind::Floor(Some(object)) = tile_data.kind {
                        return (specific_object.is_none() || specific_object == Some(object))
                            && TaskKind::Pickup.can_be_completed(tile_data)
                            && !task_indexes.contains(&index)
                            && !q_tasks
                                .iter()
                                .any(|t| !matches!(t.kind, TaskKind::Stockpile) && t.pos == index);
                        // make sure there's no task here already (excluding Stockpile tasks)
                    }
                }

                false
            });

            if let Some(index) = index {
                commands.spawn(TaskBundle::new(Task::new(
                    index,
                    TaskKind::Pickup,
                    TaskNeeds::EmptyHands,
                    tilemap_data,
                )));

                task_indexes.push(index);
            }
        }
    }
}
