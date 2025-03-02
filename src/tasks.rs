use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    utils::hashbrown::HashSet,
};
use dashmap::DashSet;
use pathfinding::directed::astar::astar;
use rand::Rng;

use crate::{
    data::{ObjectId, TileId, WORKSTATIONS},
    dwellers::Dweller,
    dwellers_needs::DwellerNeeds,
    mobs::Mob,
    tilemap::{CHUNK_SIZE, TILE_SIZE},
    tilemap_data::TilemapData,
    tiles::TilePlaced,
    ObjectSlot, SpriteLoader,
};

const Z_INDEX: f32 = 2.0;

#[derive(PartialEq, Clone, Copy, Reflect, Default, Debug)]
pub enum TaskKind {
    #[default]
    Dig,
    Smoothen,
    Harvest,
    Pickup,
    Attack,
    Stockpile,
    Build {
        result: BuildResult,
    },
    Workstation {
        amount: u32,
    },
    Walk,
    UseToSatisfyNeed,
}

impl TaskKind {
    pub fn is_valid_on_tile(self, tile: TilePlaced) -> bool {
        match self {
            TaskKind::Dig => matches!(
                tile.id,
                TileId::DirtWall | TileId::StoneWall | TileId::DungeonWall | TileId::WoodWall
            ),
            TaskKind::Smoothen => matches!(
                tile.id,
                TileId::DirtWall | TileId::StoneWall | TileId::StoneFloor
            ),
            TaskKind::Harvest => matches!(
                tile.object,
                Some(
                    ObjectId::Tree
                        | ObjectId::PalmTree
                        | ObjectId::Cactus
                        | ObjectId::TallGrass
                        | ObjectId::WheatPlant
                )
            ),
            TaskKind::Build {
                result: BuildResult::Tile(TileId::Bridge),
            } => tile.id == TileId::Water,
            TaskKind::Build { .. } => !tile.id.data().is_wall() && tile.object.is_none(),
            TaskKind::Pickup => {
                !tile.id.data().is_wall()
                    && tile
                        .object
                        .is_some_and(|object| object.data().is_carriable())
            }
            TaskKind::Attack => true,
            TaskKind::Stockpile => {
                !tile.id.data().is_wall()
                    && tile
                        .object
                        .is_none_or(|object| object.data().is_carriable())
            }
            TaskKind::Workstation { .. } => tile
                .object
                .is_some_and(|object| WORKSTATIONS.contains_key(&object)),
            TaskKind::Walk => !tile.is_blocking(),
            TaskKind::UseToSatisfyNeed => {
                matches!(tile.object, Some(ObjectId::Bed | ObjectId::Bread))
            }
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

    pub fn sprite_path(self) -> String {
        format!("tasks/{}.png", self.id())
    }
}

#[derive(PartialEq, Clone, Copy, Reflect, Debug)]
pub enum BuildResult {
    Object(ObjectId),
    Tile(TileId),
}

impl BuildResult {
    pub fn sprite_path(self) -> String {
        match self {
            BuildResult::Object(object) => object.data().sprite_path(),
            BuildResult::Tile(tile) => tile.data().sprite_path(),
        }
    }

    pub fn debug_name(self) -> String {
        match self {
            BuildResult::Object(object) => format!("{object:?}"),
            BuildResult::Tile(tile) => format!("{tile:?}"),
        }
    }
}

#[derive(Bundle)]
pub struct TaskBundle {
    pub name: Name,
    pub task: Task,
    pub needs: TaskNeeds,
    pub sprite: SpriteLoader,
    pub transform: Transform,
}

impl TaskBundle {
    pub fn new(task: Task, needs: TaskNeeds) -> Self {
        let x = task.pos.x as f32 * TILE_SIZE;
        let y = task.pos.y as f32 * TILE_SIZE;

        Self::new_inner(task, needs, x, y)
    }

    pub fn new_as_child(task: Task, needs: TaskNeeds) -> Self {
        Self::new_inner(task, needs, 0.0, 0.0)
    }

    fn new_inner(task: Task, needs: TaskNeeds, x: f32, y: f32) -> Self {
        Self {
            name: Name::new(format!("Task {:?}", task.kind)),
            needs,
            sprite: SpriteLoader {
                texture_path: task.kind.sprite_path(),
            },
            task,
            transform: Transform::from_xyz(x, y, Z_INDEX),
        }
    }
}

#[derive(Component, Reflect, PartialEq, Clone, Default, Debug)]
#[reflect(Component)]
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
    id: u64,
    pub kind: TaskKind,
    pub pos: IVec2,
    pub reachable_pathfinding: bool,
    pub reachable_positions: Vec<IVec2>,
    pub dweller: Option<Entity>,
    pub priority: i32,
}

impl MapEntities for Task {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.dweller {
            self.dweller = Some(entity_mapper.map_entity(entity));
        }
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl Task {
    pub fn new(
        pos: IVec2,
        kind: TaskKind,
        dweller: Option<Entity>,
        tilemap_data: &TilemapData,
    ) -> Self {
        let mut task = Self {
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            kind,
            pos,
            reachable_pathfinding: true,
            reachable_positions: vec![],
            dweller,
            priority: 0,
        };
        task.recompute_reachable_positions(tilemap_data);
        task
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
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
                    |p| (p.x - dweller_pos.x).abs() + (p.y - dweller_pos.y).abs(),
                    |p| *p == dweller_pos,
                )
            })
            .min_by_key(|path| path.1)
    }
}

pub fn update_unreachable_tasks(tilemap_data: Res<TilemapData>, mut q_tasks: Query<&mut Task>) {
    if tilemap_data.is_changed() {
        q_tasks.par_iter_mut().for_each(|mut task| {
            if task.reachable_positions.is_empty() || task.dweller.is_none() {
                task.recompute_reachable_positions(&tilemap_data);
            }
        });
    }
}

pub fn update_unreachable_pathfinding_tasks(mut q_tasks: Query<&mut Task>) {
    q_tasks.par_iter_mut().for_each(|mut task| {
        if !task.reachable_pathfinding {
            task.reachable_pathfinding = true;
        }
    });
}

#[derive(Event)]
pub struct TaskCompletionEvent {
    pub task: Entity,
}

pub fn event_task_completion(
    mut commands: Commands,
    mut events: EventReader<TaskCompletionEvent>,
    mut tilemap_data: ResMut<TilemapData>,
    q_mobs: Query<(Entity, &Mob, &Transform)>,
    mut q_dwellers: Query<(&mut Dweller, &mut DwellerNeeds, &Transform)>,
    mut q_tasks: Query<(Entity, &mut Task, &mut TaskNeeds, Option<&Parent>)>,
) {
    let mut rng = rand::rng();

    let mut update_tasks_pos = false;
    let mut update_stockpiles = false;
    let mut update_workstations = false;

    let tasks_positions = q_tasks
        .iter()
        .map(|(_, task, _, _)| task.pos)
        .collect::<HashSet<_>>();

    for event in events.read() {
        let Ok((entity, mut task, mut task_needs, task_parent)) = q_tasks.get_mut(event.task)
        else {
            continue;
        };

        let Some((mut dweller, mut dweller_needs, dweller_transform)) =
            task.dweller.and_then(|d| q_dwellers.get_mut(d).ok())
        else {
            continue;
        };

        let Some(tile) = tilemap_data.get(task.pos) else {
            continue;
        };

        let mut success = false;

        // just to be sure
        if !task.kind.is_valid_on_tile(tile) {
            error!("SHOULD NEVER HAPPEN: removing invalid task {task:?} on tile {tile:?}");
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let about_to_finish = match &*task_needs {
            TaskNeeds::Objects(objects) => objects.len() == 1,
            TaskNeeds::Impossible => false,
            _ => true,
        };

        // Apply task, set success to true
        // if success, TaskNeeds are handled after
        match task.kind {
            TaskKind::Dig => {
                let object = if let Some(object) = tile.object {
                    Some(object)
                } else if rng.random_bool(0.2) {
                    Some(ObjectId::Rock)
                } else {
                    None
                };

                let tile = if let Some(object) = object {
                    commands.spawn(TaskBundle::new(
                        Task::new(task.pos, TaskKind::Pickup, None, &tilemap_data),
                        TaskNeeds::EmptyHands,
                    ));

                    TileId::StoneFloor.with(object)
                } else {
                    TileId::StoneFloor.place()
                };

                tilemap_data.set(task.pos, tile);

                dweller_needs.sleep(-5);

                debug!("Dug tile at {:?}", task.pos);
                update_tasks_pos = true;
                success = true;
            }

            TaskKind::Smoothen => {
                let tile = if tile.id.data().is_wall() {
                    TileId::DungeonWall.place()
                } else if let Some(object) = tile.object {
                    TileId::DungeonFloor.with(object)
                } else {
                    TileId::DungeonFloor.place()
                };

                tilemap_data.set(task.pos, tile);

                debug!("Smoothened tile at {:?}", task.pos);
                success = true;
            }

            TaskKind::Harvest => {
                if let Some(object) = tile.object {
                    let drop_object = match object {
                        ObjectId::Tree | ObjectId::PalmTree | ObjectId::Cactus => {
                            Some(ObjectId::Wood)
                        }

                        ObjectId::TallGrass => Some(ObjectId::Seeds),

                        ObjectId::WheatPlant => {
                            dweller.object = Some(if rng.random_bool(0.7) {
                                ObjectId::Wheat
                            } else {
                                ObjectId::Seeds
                            });

                            if rng.random_bool(0.1) {
                                for (pos, tile) in tilemap_data.neighbours(task.pos) {
                                    if tile.is_floor_free() && !tasks_positions.contains(&pos) {
                                        tilemap_data.set(pos, tile.id.with(ObjectId::Farm));
                                        break;
                                    }
                                }
                            }

                            Some(ObjectId::Farm)
                        }

                        _ => None,
                    };

                    if let Some(object) = drop_object {
                        tilemap_data.set(task.pos, tile.id.with(object));

                        if object.data().is_carriable() {
                            commands.spawn(TaskBundle::new(
                                Task::new(task.pos, TaskKind::Pickup, None, &tilemap_data),
                                TaskNeeds::EmptyHands,
                            ));
                        }
                    } else {
                        tilemap_data.set(task.pos, tile.id.place());
                    }

                    dweller_needs.sleep(-2);

                    debug!("Harvested object at {:?}", task.pos);
                    update_tasks_pos = true;
                    success = true;
                }
            }

            TaskKind::Pickup => {
                if let Some(object) = tile.object {
                    tilemap_data.set(task.pos, tile.id.place());

                    match (object.data().slot(), dweller.tool, dweller.armor) {
                        (ObjectSlot::Tool, None, _) => {
                            dweller.tool = Some(object);
                            debug!("Picked up tool {:?} at {:?}", object, task.pos);
                        }

                        (ObjectSlot::Armor, _, None) => {
                            dweller.armor = Some(object);
                            debug!("Picked up armor {:?} at {:?}", object, task.pos);
                        }

                        _ => {
                            dweller.object = Some(object);
                            debug!("Picked up object {:?} at {:?}", object, task.pos);
                        }
                    }

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
                // If Build needs more than one object (!about_to_finish),
                // and is not being directly completed with the goal object,
                // do not complete the task (yet)
                //FIXME: if TaskNeeds objects have already been consumed, the task should not be completable with the goal object (as it would waste the needed objects).
                if !about_to_finish
                    && !match result {
                        BuildResult::Object(object) => dweller.object == Some(object),
                        _ => false,
                    }
                {
                    debug!("Progressing build task {:?}", task);
                } else {
                    match result {
                        BuildResult::Object(object) => {
                            tilemap_data.set(task.pos, tile.id.with(object));

                            if let Some(workstation) = WORKSTATIONS.get(&object) {
                                commands.spawn(TaskBundle::new(
                                    Task::new(
                                        task.pos,
                                        TaskKind::Workstation { amount: 1 },
                                        None,
                                        &tilemap_data,
                                    ),
                                    TaskNeeds::Objects(workstation.1.clone()),
                                ));
                            }
                        }
                        BuildResult::Tile(tile) => {
                            tilemap_data.set(task.pos, tile.place());
                        }
                    }
                }

                dweller_needs.sleep(-3);
                dweller_needs.food(-1);

                debug!("Built {:?} at {:?}", result, task.pos);
                update_tasks_pos = true;
                success = true;
            }

            TaskKind::Attack => {
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
                                    tilemap_data.set(mob_pos, loot_tile.id.with(mob.loot));

                                    commands.spawn(TaskBundle::new(
                                        Task::new(mob_pos, TaskKind::Pickup, None, &tilemap_data),
                                        TaskNeeds::EmptyHands,
                                    ));
                                } else {
                                    debug!(
                                        "Attacked mob at {:?} but loot tile is occupied",
                                        mob_pos
                                    );
                                }
                            }

                            commands.entity(entity_mob).despawn_recursive();

                            dweller_needs.sleep(-5);
                            dweller_needs.food(-5);

                            debug!("Attacked mob at {:?}", mob_transform.translation);
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
                        tilemap_data.set(task.pos, tile.id.with(object));

                        debug!("Stockpiled object at {:?}", task.pos);
                        update_tasks_pos = true;
                        success = true;
                    }
                }
            }

            TaskKind::Workstation { .. } => {
                if let Some(recipe) = tile.object.and_then(|object| WORKSTATIONS.get(&object)) {
                    if about_to_finish {
                        for (pos, tile) in tilemap_data.neighbours(task.pos) {
                            if tile.is_floor_free() && !tasks_positions.contains(&pos) {
                                tilemap_data.set(pos, tile.id.with(recipe.0));

                                if recipe.0.data().is_carriable() {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(pos, TaskKind::Pickup, None, &tilemap_data),
                                        TaskNeeds::EmptyHands,
                                    ));
                                }

                                dweller_needs.sleep(-1);
                                dweller_needs.food(-1);

                                debug!("Workstation output at {:?}", pos);
                                success = true;
                                break;
                            }
                        }
                    } else {
                        debug!("Progressing workstation task {:?}", task);
                        success = true;
                    }
                }
            }

            TaskKind::Walk => {
                success = true;
            }

            TaskKind::UseToSatisfyNeed => {
                if let Some(object) = tile.object {
                    match object {
                        ObjectId::Bread => {
                            tilemap_data.set(task.pos, tile.id.place());
                            dweller_needs.food(500);

                            debug!("Ate bread {:?}", dweller_needs);
                            success = true;
                        }

                        ObjectId::Bed => {
                            dweller_needs.sleep(100);

                            debug!("Zzzzz {:?}", dweller_needs);
                            if dweller_needs.is_fully_rested() {
                                success = true;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }

        if success {
            let mut remove_task = true;

            match *task_needs {
                TaskNeeds::Objects(ref mut objects) => {
                    if let Some(dweller_object) = dweller.object {
                        if matches!(
                            task.kind,
                            TaskKind::Build {
                                result: BuildResult::Object(build_object),
                                ..
                            } if build_object == dweller_object
                        ) {
                            dweller.object = None;
                            remove_task = true;
                        } else if let Some(i) =
                            objects.iter().position(|object| *object == dweller_object)
                        {
                            dweller.object = None;
                            objects.swap_remove(i);
                            remove_task = objects.is_empty();
                        } else {
                            error!("SHOULD NEVER HAPPEN: Dweller {} completed task TaskNeeds::Objects {:?} with object {:?} not in list", dweller.name, task.kind, dweller_object);
                        }
                    } else {
                        error!("SHOULD NEVER HAPPEN: Dweller {} completed task TaskNeeds::Objects {:?} without any object", dweller.name, task.kind);
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

            // Do not remove permanent tasks
            match task.kind {
                TaskKind::Stockpile => {
                    *task_needs = TaskNeeds::Impossible;
                    remove_task = false;
                }

                TaskKind::Workstation { ref mut amount } => {
                    if remove_task {
                        if let Some(recipe) =
                            tile.object.and_then(|object| WORKSTATIONS.get(&object))
                        {
                            *amount = amount.saturating_sub(1);
                            *task_needs = TaskNeeds::Objects(recipe.1.clone());
                        }
                        remove_task = false;
                    }
                }

                _ => {}
            }

            if remove_task {
                commands.entity(entity).despawn_recursive();
            } else {
                task.dweller = None;
            }
        }
    }

    if update_tasks_pos {
        for (_, mut task, _, _) in &mut q_tasks {
            task.recompute_reachable_positions(&tilemap_data);
        }
    }

    // Set Stockpile tasks to AnyObject if there is nothing on the tile
    if update_stockpiles {
        for (_, task, mut task_needs, _) in &mut q_tasks {
            if matches!(task.kind, TaskKind::Stockpile)
                && tilemap_data
                    .get(task.pos)
                    .is_some_and(TilePlaced::is_floor_free)
            {
                *task_needs = TaskNeeds::AnyObject;
            }
        }
    }

    // Remove Workstation tasks if the workstation is gone
    if update_workstations {
        for (entity, task, _, _) in &q_tasks {
            if matches!(task.kind, TaskKind::Workstation { .. })
                && tilemap_data
                    .get(task.pos)
                    .is_some_and(TilePlaced::is_floor_free)
            {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn update_pickups(
    par_commands: ParallelCommands,
    tilemap_data: Res<TilemapData>,
    q_tasks: Query<(Ref<Task>, Ref<TaskNeeds>)>,
    q_dwellers: Query<(Entity, &Dweller)>,
) {
    // FIXME: task.is_changed() || task_needs.is_changed() seems to always return true
    let mut updated = false;

    // Precompute existing pickup objects
    let mut existing_pickups = HashSet::new();
    for (task, task_needs) in &q_tasks {
        updated = updated || task.is_changed() || task_needs.is_changed();

        if task.kind == TaskKind::Pickup {
            if let Some(tile) = tilemap_data.get(task.pos) {
                if let Some(object) = tile.object {
                    existing_pickups.insert(object);
                }
            } else {
                error!(
                    "SHOULD NEVER HAPPEN: Pickup task at {:?} has no object",
                    task.pos
                );
            }
        }
    }

    if !updated {
        return;
    }

    // Precompute dwellers with an object, not working on a task that needs it
    let mut dwellers_candidates = HashSet::new();
    for (entity_dweller, dweller) in &q_dwellers {
        if let Some(object) = dweller.object {
            let not_working_on_task_that_needs_it =
                !q_tasks.iter().any(|(t, tn)| {
                    t.dweller == Some(entity_dweller)
                        && matches!(tn.into_inner(), TaskNeeds::Objects(objects) if objects.iter().any(|object| dweller.object == Some(*object)))
                });

            if not_working_on_task_that_needs_it {
                dwellers_candidates.insert(object);
            }
        }
    }

    let task_indexes = DashSet::new();

    q_tasks.par_iter().for_each(|(task, task_needs)| {
        // Closure result enum
        enum TryFindObjectResult {
            Wait,
            Found,
            NotFound,
        }

        if task.dweller.is_some()
            || matches!(
                task.kind,
                TaskKind::Stockpile | TaskKind::Workstation { amount: 0 }
            )
        {
            return;
        }

        if let TaskNeeds::Objects(needs_objects) = task_needs.into_inner() {
            // Closure to find an object for a task
            let try_find_object = |needs_object: &ObjectId| {
                // check if it needs a new Pickup task:
                // check for existing Pickup tasks for the required object
                // or Dwellers with the required object
                if existing_pickups.contains(needs_object)
                    || dwellers_candidates.contains(needs_object)
                {
                    return TryFindObjectResult::Wait;
                }

                // Find object: iter on stockpiles tasks containing required object, sort by distance
                let stockpiles = q_tasks
                    .iter()
                    .filter_map(|(t, _)| {
                        if matches!(t.kind, TaskKind::Stockpile)
                            && !task_indexes.contains(&t.pos)
                            && matches!(
                                tilemap_data.get(t.pos),
                                Some(TilePlaced {
                                    object: Some(o),
                                    ..
                                }) if *needs_object == o
                            )
                        {
                            const CHUNK_SIZE_SQUARED: i32 = (CHUNK_SIZE * CHUNK_SIZE) as i32;
                            let distance = t.pos.distance_squared(task.pos);
                            if distance < CHUNK_SIZE_SQUARED {
                                return Some((t.into_inner(), distance));
                            }
                        }

                        None
                    })
                    .collect::<Vec<_>>();

                // Get closest stockpile
                let stockpile = stockpiles.into_iter().min_by_key(|(_, distance)| *distance);

                if let Some((Task { pos, .. }, _)) = stockpile {
                    debug!("Found object {needs_object:?} at {pos:?} for {task:?}");

                    par_commands.command_scope(|mut commands| {
                        commands.spawn(TaskBundle::new(
                            Task::new(*pos, TaskKind::Pickup, None, &tilemap_data),
                            TaskNeeds::EmptyHands,
                        ));
                    });

                    task_indexes.insert(pos);
                    return TryFindObjectResult::Found;
                }

                TryFindObjectResult::NotFound
            };

            match task.kind {
                TaskKind::Build {
                    result: BuildResult::Object(object),
                } => {
                    // for Build tasks, check if the goal object is directly available
                    match try_find_object(&object) {
                        TryFindObjectResult::Found | TryFindObjectResult::Wait => return,
                        TryFindObjectResult::NotFound => {}
                    }
                }
                _ => {}
            }

            for needs_object in needs_objects {
                try_find_object(needs_object);
            }
        }
    });
}
