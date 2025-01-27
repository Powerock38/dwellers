use std::collections::BinaryHeap;

use bevy::{
    prelude::*,
    sprite::Anchor,
    utils::{HashMap, HashSet},
};
use rand::{seq::IndexedRandom, Rng};

use crate::{
    data::ObjectId,
    random_text::{generate_word, NAMES},
    tasks::{BuildResult, Task, TaskCompletionEvent, TaskKind, TaskNeeds},
    tilemap::TILE_SIZE,
    tilemap_data::TilemapData,
    LoadChunk, SpriteLoader, UnloadChunk, CHUNK_SIZE,
};

const LOAD_CHUNKS_RADIUS: i32 = 1;

const SPEED: f32 = 100.0;
const Z_INDEX: f32 = 10.0;

#[derive(Event)]
pub struct SpawnDwellersOnChunk(pub IVec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Dweller {
    pub name: String,
    speed: f32,
    pub move_queue: Vec<IVec2>, // next move is at the end
    pub object: Option<ObjectId>,
    pub tool: Option<ObjectId>,
    pub armor: Option<ObjectId>,
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
        let Some(spawn_pos) = TilemapData::find_from_center(
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
                Dweller {
                    name: name.to_string(),
                    speed: SPEED,
                    ..default()
                },
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

        let index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        // Check if dweller has a task assigned in all tasks
        let task = q_tasks
            .iter_mut()
            .sort::<&Task>()
            .find(|(_, task, _)| task.dweller == Some(entity));

        if let Some((entity_task, mut task, _)) = task {
            if task.reachable_positions.iter().any(|pos| *pos == index) {
                // Reached task location
                ev_task_completion.send(TaskCompletionEvent { task: entity_task });
            } else {
                // Task moved, try to pathfind again
                if let Some(path) = task.pathfind(index, &tilemap_data) {
                    debug!("Dweller {} can re-pathfind to {:?}", dweller.name, task);
                    dweller.move_queue = path.0;
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

    let mut dwellers = q_dwellers
        .iter_mut()
        .filter_map(|(entity, dweller, transform)| {
            if assigned_dwellers.contains(&entity) {
                return None;
            }
            let index = IVec2::new(
                (transform.translation.x / TILE_SIZE) as i32,
                (transform.translation.y / TILE_SIZE) as i32,
            );
            Some((entity, dweller, index))
        })
        .collect::<Vec<_>>();

    let mut tasks = q_tasks
        .iter_mut()
        .filter(|(_, task, _)| {
            task.dweller.is_none()
                && !task.reachable_positions.is_empty()
                && task.reachable_pathfinding
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

        match task.kind {
            TaskKind::Workstation { amount: 0 } => continue,
            _ => {}
        }

        match task_needs {
            TaskNeeds::Nothing => {}
            TaskNeeds::EmptyHands => {
                if dweller.object.is_some() {
                    continue;
                }
            }
            TaskNeeds::Objects(objects) => match dweller.object {
                None => continue,
                Some(dweller_object) => {
                    if !objects.iter().any(|object| *object == dweller_object)
                        && !matches!(
                            task.kind,
                            TaskKind::Build {
                                result: BuildResult::Object(build_object),
                                ..
                            } if build_object == dweller_object
                        )
                    {
                        continue;
                    }
                }
            },
            TaskNeeds::AnyObject => {
                if dweller.object.is_none() {
                    continue;
                }
            }
            TaskNeeds::Impossible => {
                continue;
            }
        }

        // Try pathfinding to task
        if let Some(path) = task.pathfind(*dweller_pos, &tilemap_data) {
            task.dweller = Some(*dweller_entity);
            dweller.move_queue = path.0;

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

            if direction.length() < dweller.speed * time.delta_secs() {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
                dweller.move_queue.pop();
            } else {
                let dir = direction.normalize();
                transform.translation.x += dir.x * dweller.speed * time.delta_secs();
                transform.translation.y += dir.y * dweller.speed * time.delta_secs();

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
        let index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        // Load new chunks if needed
        let (chunk_index, _) = TilemapData::index_to_chunk(index);

        for dx in -LOAD_CHUNKS_RADIUS..=LOAD_CHUNKS_RADIUS {
            for dy in -LOAD_CHUNKS_RADIUS..=LOAD_CHUNKS_RADIUS {
                let chunk_index = chunk_index + IVec2::new(dx, dy);

                if !sent_event_for.contains(&chunk_index) {
                    ev_load_chunk.send(LoadChunk(chunk_index));
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
            ev_unload_chunk.send(UnloadChunk(*chunk_index));
        });

    // Decrease chunk TTL
    for ttl in chunks_ttl.values_mut() {
        *ttl -= 1;
    }

    chunks_ttl.retain(|_, ttl| *ttl > 0);
}
