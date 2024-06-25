use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};
use bevy_entitiles::tilemap::map::TilemapStorage;
use pathfinding::directed::astar::astar;
use rand::Rng;

use crate::{
    data::{ObjectId, TileId, WORKSTATIONS},
    dwellers::Dweller,
    extract_ok,
    mobs::Mob,
    tilemap::{TilemapData, TILE_SIZE},
    tiles::TilePlaced,
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
    },
    Workstation,
    GoThere,
}

impl TaskKind {
    pub fn is_valid_on_tile(self, tile: TilePlaced) -> bool {
        let tile_id = tile.id;
        match self {
            TaskKind::Dig => {
                tile_id == TileId::DirtWall
                    || tile_id == TileId::StoneWall
                    || tile_id == TileId::DungeonWall
            }
            TaskKind::Smoothen => {
                tile_id == TileId::DirtWall
                    || tile_id == TileId::StoneWall
                    || tile_id == TileId::StoneFloor
            }
            TaskKind::Harvest => {
                matches!(
                    tile.object,
                    Some(ObjectId::Tree | ObjectId::TallGrass | ObjectId::WheatPlant)
                )
            }
            TaskKind::Bridge => tile_id == TileId::Water,
            TaskKind::Build { .. } => tile.object.is_none(),
            TaskKind::Pickup => {
                !tile_id.data().is_wall()
                    && tile
                        .object
                        .map_or(false, |object| object.data().is_carriable())
            }
            TaskKind::Hunt => true,
            TaskKind::Stockpile => {
                !tile_id.data().is_wall()
                    && tile
                        .object
                        .map_or(true, |object| object.data().is_carriable())
            }
            TaskKind::Workstation => tile
                .object
                .map_or(false, |object| WORKSTATIONS.contains_key(&object)),
            TaskKind::GoThere => !tile.is_blocking(),
        }
    }

    pub fn id(self) -> String {
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
    Object(ObjectId),
    Tile(TileId),
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

#[derive(PartialEq, Clone, Reflect, Default, Debug)]
pub enum TaskNeeds {
    #[default]
    Nothing,
    EmptyHands,
    Objects(Vec<ObjectId>),
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
        if let Some(tile) = tilemap_data.get(pos) {
            let will_build_wall = if let TaskKind::Build {
                result: BuildResult::Tile(tile_id),
                ..
            } = self.kind
            {
                tile_id.data().is_wall()
            } else {
                false
            };

            if !tile.is_blocking() && !will_build_wall {
                return vec![pos];
            }
        }

        tilemap_data.non_blocking_neighbours_pos(pos, true)
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
    let mut update_workstations = false;

    for event in events.read() {
        let Ok((entity, mut task, task_parent)) = q_tasks.get_mut(event.task) else {
            continue;
        };

        let Some((mut dweller, dweller_transform)) =
            task.dweller.and_then(|d| q_dwellers.get_mut(d).ok())
        else {
            continue;
        };

        let Some(tile) = tilemap_data.get(task.pos) else {
            continue;
        };

        let mut rng = rand::thread_rng();

        let mut success = false;

        if !task.kind.is_valid_on_tile(tile) {
            error!("SHOULD NEVER HAPPEN: removing invalid task {task:?} on tile {tile:?}");
            commands.entity(entity).despawn_recursive();
            continue;
        }

        match task.kind {
            TaskKind::Dig => {
                let object = if let Some(object) = tile.object {
                    Some(object)
                } else if rng.gen_bool(0.2) {
                    Some(ObjectId::Rock)
                } else {
                    None
                };

                let tile = if let Some(object) = object {
                    commands.spawn(TaskBundle::new(Task::new(
                        task.pos,
                        TaskKind::Pickup,
                        TaskNeeds::EmptyHands,
                        &tilemap_data,
                    )));

                    TileId::StoneFloor.with(object)
                } else {
                    TileId::StoneFloor.empty()
                };

                tile.set_at(task.pos, &mut commands, &mut tilemap, &mut tilemap_data);

                debug!("Dug tile at {:?}", task.pos);
                update_tasks_pos = true;
                success = true;
            }

            TaskKind::Smoothen => {
                let tile = if tile.id.data().is_wall() {
                    TileId::DungeonWall.empty()
                } else if let Some(object) = tile.object {
                    TileId::DungeonFloor.with(object)
                } else {
                    TileId::DungeonFloor.empty()
                };

                tile.set_at(task.pos, &mut commands, &mut tilemap, &mut tilemap_data);

                debug!("Smoothened tile at {:?}", task.pos);
                success = true;
            }

            TaskKind::Harvest => match tile.object {
                Some(object) => {
                    let drop_object = match object {
                        ObjectId::Tree => {
                            if rng.gen_bool(0.3) {
                                Some(ObjectId::Wood)
                            } else {
                                None
                            }
                        }

                        ObjectId::TallGrass => Some(ObjectId::Seeds),

                        ObjectId::WheatPlant => {
                            dweller.object = Some(ObjectId::Wheat);

                            Some(ObjectId::Farm)
                        }

                        _ => None,
                    };

                    if let Some(object) = drop_object {
                        tile.id.with(object).set_at(
                            task.pos,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        if object.data().is_carriable() {
                            commands.spawn(TaskBundle::new(Task::new(
                                task.pos,
                                TaskKind::Pickup,
                                TaskNeeds::EmptyHands,
                                &tilemap_data,
                            )));
                        }
                    } else {
                        tile.id.empty().set_at(
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
                _ => {}
            },

            TaskKind::Bridge => {
                TileId::BridgeFloor.empty().set_at(
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
                if let Some(object) = tile.object {
                    tile.id.empty().set_at(
                        task.pos,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );

                    dweller.object = Some(object);

                    debug!("Picked up object at {:?}", task.pos);
                    if object.data().is_blocking() {
                        update_tasks_pos = true;
                    }
                    update_stockpiles = true;
                    if WORKSTATIONS.contains_key(&object) {
                        update_workstations = true;
                    }
                    success = true;
                }
            }

            TaskKind::Build { result } => {
                match &task.needs {
                    TaskNeeds::Objects(objects) if objects.len() > 1 => {
                        debug!("Progressing build task {:?}", task);
                    }

                    _ => match result {
                        BuildResult::Object(object) => {
                            tile.id.with(object).set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );

                            if let Some(workstation) = WORKSTATIONS.get(&object) {
                                commands.spawn(TaskBundle::new(Task::new(
                                    task.pos,
                                    TaskKind::Workstation,
                                    TaskNeeds::Objects(workstation.1.clone()),
                                    &tilemap_data,
                                )));
                            }
                        }
                        BuildResult::Tile(tile) => {
                            tile.empty().set_at(
                                task.pos,
                                &mut commands,
                                &mut tilemap,
                                &mut tilemap_data,
                            );
                        }
                    },
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
                            if let Some(loot_tile) = tilemap_data.get(mob_pos) {
                                if loot_tile.object.is_none() {
                                    loot_tile.id.with(mob.loot).set_at(
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
                if tile.object.is_none() {
                    if let Some(object) = dweller.object {
                        tile.id.with(object).set_at(
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

            TaskKind::Workstation => {
                if let Some(workstation) = tile.object {
                    if let Some(recipe) = WORKSTATIONS.get(&workstation) {
                        for (pos, tile) in tilemap_data.neighbours(task.pos) {
                            if tile.is_floor_free() {
                                // TODO: ensure there is no task at pos

                                tile.id.with(recipe.0).set_at(
                                    pos,
                                    &mut commands,
                                    &mut tilemap,
                                    &mut tilemap_data,
                                );

                                if recipe.0.data().is_carriable() {
                                    commands.spawn(TaskBundle::new(Task::new(
                                        pos,
                                        TaskKind::Pickup,
                                        TaskNeeds::EmptyHands,
                                        &tilemap_data,
                                    )));
                                }

                                debug!("Workstation output at {:?}", pos);
                                success = true;
                                break;
                            }
                        }
                    }
                }
            }

            TaskKind::GoThere => {
                success = true;
            }
        }

        if success {
            let mut remove_task = true;

            let kind = task.kind;
            match &mut task.needs {
                TaskNeeds::Objects(ref mut objects) => {
                    if let Some(dweller_object) = dweller.object {
                        if matches!(
                            kind,
                            TaskKind::Build {
                                result: BuildResult::Object(build_object),
                                ..
                            } if build_object == dweller_object
                        ) {
                            dweller.object = None;
                        } else if let Some(i) =
                            objects.iter().position(|object| *object == dweller_object)
                        {
                            dweller.object = None;
                            objects.swap_remove(i);
                            remove_task = objects.is_empty();
                        } else {
                            error!("SHOULD NEVER HAPPEN: Dweller {} completed task TaskNeeds::Objects {:?} with object {:?} not in list", dweller.name, kind, dweller_object);
                        }
                    } else {
                        error!("SHOULD NEVER HAPPEN: Dweller {} completed task TaskNeeds::Objects {:?} without any object", dweller.name, kind);
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
                    remove_task = false;
                    error!(
                        "SHOULD NEVER HAPPEN: Dweller {} completed impossible task {:?}",
                        dweller.name, task
                    );
                }

                TaskNeeds::Nothing | TaskNeeds::EmptyHands => {}
            }

            // Permanent tasks
            match task.kind {
                TaskKind::Stockpile => {
                    task.needs = TaskNeeds::Impossible;
                    remove_task = false;
                }

                TaskKind::Workstation => {
                    if remove_task {
                        if let Some(workstation) = tile.object {
                            if let Some(recipe) = WORKSTATIONS.get(&workstation) {
                                task.needs = TaskNeeds::Objects(recipe.1.clone());
                            }
                        }
                        remove_task = false;
                    }
                }

                _ => {}
            }

            if remove_task {
                commands.entity(entity).despawn();
            } else {
                task.dweller = None;
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

    // Set Stockpile tasks to AnyObject if there is nothing on the tile
    if update_stockpiles {
        for (_, mut task, _) in &mut q_tasks {
            if matches!(task.kind, TaskKind::Stockpile)
                && tilemap_data
                    .get(task.pos)
                    .map_or(false, TilePlaced::is_floor_free)
            {
                task.needs = TaskNeeds::AnyObject;
            }
        }
    }

    // Remove Workstation tasks if the workstation is gone
    if update_workstations {
        for (entity, task, _) in &mut q_tasks {
            if matches!(task.kind, TaskKind::Workstation { .. })
                && tilemap_data
                    .get(task.pos)
                    .map_or(false, TilePlaced::is_floor_free)
            {
                commands.entity(entity).despawn_recursive();
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
        if matches!(task.kind, TaskKind::Stockpile) {
            continue;
        }

        let (specific_objects, needs_objects) = match &task.needs {
            TaskNeeds::Objects(objects) => (Some(objects), true),
            TaskNeeds::AnyObject => (None, true),
            _ => (None, false),
        };

        if needs_objects {
            // check if it needs a new Pickup task: check for already existing Pickup tasks for the required object
            if q_tasks.iter().any(|t| {
                t.kind == TaskKind::Pickup
                && t.dweller.is_none() // not being worked on
                    && tilemap_data.get(t.pos).is_some_and(|tile| {
                        if let Some(object) = tile.object {
                            if let Some(objects) = specific_objects {
                                return objects.iter().any(|o| object == *o);
                            }

                            return true;
                        }

                        error!("SHOULD NEVER HAPPEN: Pickup task at {:?} has no object", t.pos);
                        false
                    })
            }) || // or Dwellers with the required object
            q_dwellers.iter().any(|(entity_dweller, dweller)| {
                let has_object = match (specific_objects, dweller.object) {
                    (Some(objects), Some(dweller_object)) => objects.iter().any(|object| *object == dweller_object),
                    (None, _) => true,
                    _ => false,
                };

                let not_working_on_task_that_needs_it =
                    !q_tasks.iter().any(|t| {
                        t.dweller == Some(entity_dweller) && matches!(&t.needs, TaskNeeds::Objects(objects) if objects.iter().any(|object| dweller.object.map_or(false, |dweller_object| dweller_object == *object) ))
                    });

                has_object && not_working_on_task_that_needs_it
            }) {
                continue;
            }

            // Find object: search around task.pos
            let index = TilemapData::find_from_center(task.pos, |index| {
                if let Some(tile) = tilemap_data.get(index) {
                    if let Some(object) = tile.object {
                        let has_object = match (specific_objects, object) {
                            (Some(objects), object) => objects.iter().any(|o| *o == object),
                            (None, _) => true,
                        };

                        return has_object
                            && TaskKind::Pickup.is_valid_on_tile(tile)
                            && !task_indexes.contains(&index)
                            // make sure there's no task here already (excluding Stockpile tasks)
                            && !q_tasks
                                .iter()
                                .any(|t| !matches!(t.kind, TaskKind::Stockpile) && t.pos == index);
                    }
                }

                false
            });

            if let Some(index) = index {
                info!("Found object at {index:?} for {task:?}");

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
