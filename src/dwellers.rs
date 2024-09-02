use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
use rand::{seq::SliceRandom, Rng};

use crate::{
    data::ObjectId,
    random_text::{generate_word, NAMES},
    tasks::{BuildResult, Task, TaskCompletionEvent, TaskKind, TaskNeeds},
    tilemap::{TilemapData, TILE_SIZE},
    LoadChunk, SpriteLoaderBundle, UnloadChunk, CHUNK_SIZE,
};

const LOAD_CHUNKS_RADIUS: i32 = 1;

const SPEED: f32 = 100.0;
const Z_INDEX: f32 = 10.0;

#[derive(Event)]
pub struct SpawnDwellersOnChunk(pub IVec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Dweller {
    pub name: String,
    speed: f32,
    move_queue: Vec<IVec2>, // next move is at the end
    pub object: Option<ObjectId>,
}

#[derive(Bundle)]
pub struct DwellerBundle {
    dweller: Dweller,
    sprite: SpriteLoaderBundle,
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
        let mut rng = rand::thread_rng();

        for _ in 0..nb_dwellers {
            let mut name = generate_word(&NAMES, &mut rng);
            name.get_mut(0..1).unwrap().make_ascii_uppercase();

            let sprite_i = rng.gen_range(1..=4);

            commands
                .spawn(DwellerBundle {
                    sprite: SpriteLoaderBundle::new(
                        format!("sprites/dweller{sprite_i}.png"),
                        spawn_pos.x as f32 * TILE_SIZE,
                        spawn_pos.y as f32 * TILE_SIZE,
                        Z_INDEX,
                    ),
                    dweller: Dweller {
                        name: name.to_string(),
                        speed: SPEED,
                        move_queue: vec![],
                        object: None,
                    },
                })
                .with_children(|c| {
                    c.spawn(Text2dBundle {
                        text: Text::from_section(
                            name,
                            TextStyle {
                                font_size: 16.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        text_anchor: Anchor::BottomCenter,
                        transform: Transform::from_scale(Vec3::splat(0.5))
                            .with_translation(Vec3::new(TILE_SIZE / 2.0, TILE_SIZE, 1.0)),
                        ..default()
                    });
                });
        }
    }
}

pub fn update_dwellers(
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
    tilemap_data: Res<TilemapData>,
    mut q_tasks: Query<(Entity, &mut Task)>,
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
            .find(|(_, task)| task.dweller == Some(entity));

        if let Some((entity_task, mut task)) = task {
            if task.reachable_positions.iter().any(|pos| *pos == index) {
                // Reached task location
                ev_task_completion.send(TaskCompletionEvent { task: entity_task });
            } else {
                // Task moved, try to pathfind again
                let path = task.pathfind(index, &tilemap_data);

                if let Some(path) = path {
                    info!("Dweller {} can re-pathfind to {:?}", dweller.name, task);
                    dweller.move_queue = path.0;
                } else {
                    info!("Dweller {} give up {:?}", dweller.name, task);
                    task.dweller = None;
                }
            }

            continue;
        }

        // Get a new task
        // FIXME: dwellers first in the loop can "steal" a task far away from them from a dweller that is closer
        let task_path = q_tasks
            .iter_mut()
            .filter_map(|(_, mut task)| {
                if task.dweller.is_none()
                    && !task.reachable_positions.is_empty()
                    && task.reachable_pathfinding
                {
                    match &task.needs {
                        TaskNeeds::Nothing => {}
                        TaskNeeds::EmptyHands => {
                            if dweller.object.is_some() {
                                return None;
                            }
                        }
                        TaskNeeds::Objects(objects) => match dweller.object {
                            None => return None,
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
                    let path = task.pathfind(index, &tilemap_data);

                    if let Some(path) = path {
                        return Some((task, path));
                    }

                    task.reachable_pathfinding = false;
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
            debug!("Dweller {} got task {task:?}", dweller.name);

            task.dweller = Some(entity);
            dweller.move_queue = path;

            continue;
        }

        // Wander around
        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.2) {
            let directions = tilemap_data.non_blocking_neighbours_pos(index, true);

            if let Some(direction) = directions.choose(&mut rng) {
                dweller.move_queue.push(*direction);
            }
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
