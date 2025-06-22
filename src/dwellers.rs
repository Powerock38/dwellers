use std::collections::BinaryHeap;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    sprite::Anchor,
};
use rand::{seq::IndexedRandom, Rng};

use crate::{
    data::ObjectId,
    objects::ObjectSlot,
    random_text::{generate_word, NAMES},
    tasks::{BuildResult, Task, TaskCompletionEvent, TaskKind, TaskNeeds},
    tilemap::TILE_SIZE,
    tilemap_data::TilemapData,
    utils::transform_to_index,
    LoadChunk, SpriteLoader, UnloadChunk, CHUNK_SIZE,
};

const LOAD_CHUNKS_RADIUS: i32 = 1;

const Z_INDEX: f32 = 10.0;

const SPEED: f32 = 120.0;
const SPEED_MIN: f32 = 0.3;

pub const NEEDS_MAX: u32 = 1000;

const HEALTH_BASE: u32 = 10;

#[derive(Event)]
pub struct SpawnDwellersOnChunk(pub IVec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Dweller {
    pub name: String,
    pub move_queue: Vec<IVec2>, // next move is at the end
    pub object: Option<ObjectId>,
    pub tool: Option<ObjectId>,
    pub armor: Option<ObjectId>,
    pub health: u32,
    pub food: u32,
    pub sleep: u32,
    pub cached_speed_ratio: f32,
}

impl Dweller {
    pub fn new(name: String) -> Self {
        Self {
            name,
            move_queue: Vec::new(),
            object: None,
            tool: None,
            armor: None,
            health: HEALTH_BASE,
            food: NEEDS_MAX,
            sleep: NEEDS_MAX,
            cached_speed_ratio: 1.0,
        }
    }

    pub fn can_do(&self, task_kind: TaskKind, task_needs: &TaskNeeds) -> bool {
        match task_kind {
            TaskKind::Workstation { amount: 0 } => return false,
            _ => {}
        }

        match task_needs {
            TaskNeeds::Nothing => {}
            TaskNeeds::EmptyHands => {
                if self.object.is_some() {
                    return false;
                }
            }
            TaskNeeds::Objects(objects) => match self.object {
                None => return false,
                Some(dweller_object) => {
                    if !objects.contains(&dweller_object)
                        && !matches!(
                            task_kind,
                            TaskKind::Build {
                                result: BuildResult::Object(build_object),
                                ..
                            } if build_object == dweller_object
                        )
                    {
                        return false;
                    }
                }
            },
            TaskNeeds::AnyObject => {
                if self.object.is_none() {
                    return false;
                }
            }
            TaskNeeds::Impossible => {
                return false;
            }
        }

        true
    }

    pub fn max_health(&self) -> u32 {
        HEALTH_BASE
            + self.armor.map_or(0, |a| match a.data().slot() {
                ObjectSlot::Armor(hp) => *hp,
                _ => 0,
            })
    }

    pub fn health(&mut self, x: i32) {
        self.health = self.health.saturating_add_signed(x).min(self.max_health());
        self.compute_speed_ratio();
    }

    pub fn food(&mut self, x: i32) {
        self.food = self.food.saturating_add_signed(x).min(NEEDS_MAX);

        if self.food == 0 {
            self.health(-1);
        }

        self.compute_speed_ratio();
    }

    pub fn sleep(&mut self, x: i32) {
        self.sleep = self.sleep.saturating_add_signed(x).min(NEEDS_MAX);

        if self.sleep == 0 {
            self.health(-1);
        }

        self.compute_speed_ratio();
    }

    fn compute_speed_ratio(&mut self) {
        let health_ratio = self.health as f32 / self.max_health() as f32;
        let health_speed = 1.0 - (health_ratio - 1.0).abs().powi(2);

        let food_ratio = self.food as f32 / NEEDS_MAX as f32;
        let food_speed = 1.0 - (food_ratio - 1.0).abs().powi(5);

        let sleep_ratio = self.sleep as f32 / NEEDS_MAX as f32;
        let sleep_speed = 1.0 - (sleep_ratio - 1.0).abs().powi(3);

        self.cached_speed_ratio = health_speed.min(food_speed.min(sleep_speed).max(SPEED_MIN));
    }

    pub fn speed_ratio(&self) -> f32 {
        self.cached_speed_ratio
    }

    pub fn is_fully_rested(&self) -> bool {
        self.sleep == NEEDS_MAX
    }
}

#[derive(Resource, Default)]
pub struct DwellersSelected {
    list: Vec<Entity>,
    i: usize,
}

impl DwellersSelected {
    pub fn next(&mut self) -> Option<Entity> {
        if self.list.is_empty() {
            return None;
        }

        let entity = self.list[self.i];
        self.i = (self.i + 1) % self.list.len();

        Some(entity)
    }

    pub fn reset(&mut self) {
        self.list.clear();
        self.i = 0;
    }

    pub fn add(&mut self, entity: Entity) {
        if !self.list.contains(&entity) {
            self.list.push(entity);
        }
    }

    pub fn list(&self) -> &[Entity] {
        &self.list
    }
}

pub fn spawn_dwellers(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut ev_spawn: EventReader<SpawnDwellersOnChunk>,
) {
    for SpawnDwellersOnChunk(chunk_index) in ev_spawn.read() {
        let Some(spawn_pos) = TilemapData::find_from_center_chunk_size(
            TilemapData::local_index_to_global(*chunk_index, IVec2::splat(CHUNK_SIZE as i32 / 2)),
            |index| {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let index = index + IVec2::new(dx, dy);

                        let Some(tile) = tilemap_data.get(index) else {
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
            error!("No valid spawn position found for dwellers");
            return;
        };

        let nb_dwellers = 10;
        let mut rng = rand::rng();

        for _ in 0..nb_dwellers {
            let mut name = generate_word(&NAMES, &mut rng);
            name.get_mut(0..1).unwrap().make_ascii_uppercase();

            let sprite_i = rng.random_range(1..=4);

            commands.spawn((
                Dweller::new(name),
                SpriteLoader {
                    texture_path: format!("sprites/dweller{sprite_i}.png"),
                },
                Transform::from_xyz(
                    spawn_pos.x as f32 * TILE_SIZE,
                    spawn_pos.y as f32 * TILE_SIZE,
                    Z_INDEX,
                ),
            ));
        }
    }
}

pub fn spawn_dwellers_name(
    mut commands: Commands,
    q_dwellers: Query<(&Dweller, Entity), Added<Dweller>>,
) {
    for (dweller, entity) in &q_dwellers {
        commands.entity(entity).with_child((
            Text2d::new(dweller.name.clone()),
            TextFont::from_font_size(16.0),
            TextColor(Color::WHITE),
            Transform::from_scale(Vec3::splat(0.5)).with_translation(Vec3::new(
                TILE_SIZE / 2.0,
                TILE_SIZE,
                2.0,
            )),
            Anchor::BottomCenter,
        ));
    }
}

pub fn update_dwellers(
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    tilemap_data: Res<TilemapData>,
    mut q_tasks: Query<(Entity, &mut Task, &TaskNeeds)>,
    mut ev_task_completion: EventWriter<TaskCompletionEvent>,
) {
    for (entity, mut dweller, transform) in &mut q_dwellers {
        if !dweller.move_queue.is_empty() {
            continue;
        }

        let index = transform_to_index(transform);

        // Check if dweller has a task assigned in all tasks
        let task = q_tasks
            .iter_mut()
            .sort::<&Task>()
            .find(|(_, task, task_needs)| {
                task.dweller == Some(entity) && dweller.can_do(task.kind, task_needs)
            });

        if let Some((entity_task, mut task, _)) = task {
            if task.reachable_positions.contains(&index) {
                // Reached task location
                ev_task_completion.write(TaskCompletionEvent { task: entity_task });
            } else {
                // Task moved, try to pathfind again
                if let Some(path) = task.pathfind(index, &tilemap_data) {
                    debug!("Dweller {} can re-pathfind to {:?}", dweller.name, task);
                    dweller.move_queue = path;
                } else {
                    info!("Dweller {} gives up {:?}", dweller.name, task);
                    task.dweller = None;
                }
            }

            continue;
        }

        // Else, wander around
        let mut rng = rand::rng();

        if rng.random_bool(0.2) {
            let directions = tilemap_data.non_blocking_neighbours_pos(index, true);

            if let Some(direction) = directions.choose(&mut rng) {
                dweller.move_queue.push(*direction);
            }
        }
    }
}

pub fn assign_tasks_to_dwellers(
    tilemap_data: Res<TilemapData>,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    mut q_tasks: Query<(Entity, &mut Task, &TaskNeeds)>,
) {
    // Collect unassigned dwellers and tasks
    let assigned_dwellers: HashSet<_> = q_tasks
        .iter()
        .filter_map(|(_, task, _)| task.dweller)
        .collect();

    let mut tasks = q_tasks
        .iter_mut()
        .filter(|(_, task, _)| {
            task.dweller.is_none()
                && !task.reachable_positions.is_empty()
                && task.reachable_pathfinding
                // Do not assign tasks that will produce a blocking object where a dweller is standing
                && (!matches!(task.kind, TaskKind::Build { result } if result.is_blocking())
                    || !q_dwellers
                        .iter()
                        .any(|(_, _, t)| task.pos == transform_to_index(t)))
        })
        .collect::<Vec<_>>();

    let mut dwellers = q_dwellers
        .iter_mut()
        .filter_map(|(entity, dweller, transform)| {
            if assigned_dwellers.contains(&entity) {
                return None;
            }
            let index = transform_to_index(transform);
            Some((entity, dweller, index))
        })
        .collect::<Vec<_>>();

    // Compute all distances
    let mut heap = BinaryHeap::new();

    for (dweller_i, (_, _, dweller_pos)) in dwellers.iter().enumerate() {
        for (task_i, (_, task, _)) in tasks.iter().enumerate() {
            let distance = (dweller_pos.x - task.pos.x).abs() + (dweller_pos.y - task.pos.y).abs();
            heap.push((task.priority, -distance, dweller_i, task_i));
        }
    }

    let mut assigned_dwellers = HashSet::new();
    let mut assigned_tasks = HashSet::new();

    // Process the heap until it is empty or all tasks/dwellers are assigned
    while let Some((_, _, dweller_i, task_i)) = heap.pop() {
        if assigned_dwellers.contains(&dweller_i) || assigned_tasks.contains(&task_i) {
            continue;
        }

        let (_, task, task_needs) = &mut tasks[task_i];
        let (dweller_entity, dweller, dweller_pos) = &mut dwellers[dweller_i];

        if !dweller.can_do(task.kind, task_needs) {
            continue;
        }

        // Try pathfinding to task
        if let Some(path) = task.pathfind(*dweller_pos, &tilemap_data) {
            task.dweller = Some(*dweller_entity);
            dweller.move_queue = path;

            assigned_dwellers.insert(dweller_i);
            assigned_tasks.insert(task_i);
            debug!("Dweller {} got task {:?}", dweller.name, task);
        } else {
            task.reachable_pathfinding = false;
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

            let speed = SPEED * dweller.speed_ratio() * time.delta_secs();

            if direction.length() < speed {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                dweller.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * speed;
                transform.translation.y += dir.y * speed;

                sprite.flip_x = dir.x < 0.0;
            }
        }
    }
}

pub fn update_dwellers_load_chunks(
    q_dwellers: Query<&Transform, With<Dweller>>,
    tilemap_data: Res<TilemapData>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_unload_chunk: EventWriter<UnloadChunk>,
    mut chunks_ttl: Local<HashMap<IVec2, u32>>,
) {
    let mut sent_event_for = vec![];

    for transform in &q_dwellers {
        let index = transform_to_index(transform);

        // Load new chunks if needed
        let (chunk_index, _) = TilemapData::index_to_chunk(index);

        for dx in -LOAD_CHUNKS_RADIUS..=LOAD_CHUNKS_RADIUS {
            for dy in -LOAD_CHUNKS_RADIUS..=LOAD_CHUNKS_RADIUS {
                let chunk_index = chunk_index + IVec2::new(dx, dy);

                if !sent_event_for.contains(&chunk_index) {
                    ev_load_chunk.write(LoadChunk(chunk_index));
                    sent_event_for.push(chunk_index);
                    chunks_ttl.insert(chunk_index, 10);
                }
            }
        }
    }

    if q_dwellers.is_empty() {
        return;
    }

    // Unload chunks without dwellers
    tilemap_data
        .chunks
        .keys()
        .filter(|chunk_index| {
            !sent_event_for.contains(chunk_index) && !chunks_ttl.contains_key(*chunk_index)
        })
        .for_each(|chunk_index| {
            ev_unload_chunk.write(UnloadChunk(*chunk_index));
        });

    // Decrease chunk TTL
    for ttl in chunks_ttl.values_mut() {
        *ttl -= 1;
    }

    chunks_ttl.retain(|_, ttl| *ttl > 0);
}
