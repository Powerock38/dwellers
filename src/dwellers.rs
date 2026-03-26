use bevy::{platform::collections::HashSet, prelude::*, sprite::Anchor};
use rand::{Rng, seq::IndexedRandom};
use uuid::Uuid;

use crate::{
    BuildResult, CHUNK_SIZE, ObjectSlot, SaveScoped, SpriteLoader, TILE_SIZE, Task,
    TaskCompletionEvent, TaskKind, TaskNeeds, TilemapData,
    data::ObjectId,
    despawn_dweller_hover,
    mobs::Mob,
    observe_dweller_hover,
    random_text::{NAMES, generate_word},
    tasks::TaskBundle,
    utils::transform_to_pos,
    zones::ZoneMap,
};

const Z_INDEX: f32 = 10.0;

const SPEED: f32 = 120.0;
const SPEED_MIN: f32 = 0.3;

pub const NEEDS_MAX: u32 = 1000;

const HEALTH_BASE: u32 = 10;

const DWELLER_DETECTION_TILE_RADIUS: i32 = 15;

#[derive(Message)]
pub struct SpawnDwellersOnChunk(pub IVec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Name::new("dweller"), SaveScoped)]
pub struct Dweller {
    pub uuid: Uuid,
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
            uuid: Uuid::new_v4(),
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
    mut ev_spawn: MessageReader<SpawnDwellersOnChunk>,
) {
    for SpawnDwellersOnChunk(chunk_pos) in ev_spawn.read() {
        let Some(spawn_pos) = TilemapData::find_from_center_chunk_size(
            TilemapData::local_pos_to_global(*chunk_pos, IVec2::splat(CHUNK_SIZE as i32 / 2)),
            |pos| {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let neigh_pos = pos + IVec2::new(dx, dy);

                        let Some(tile) = tilemap_data.get(neigh_pos) else {
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

            commands
                .spawn((
                    Dweller::new(name),
                    SpriteLoader {
                        texture_path: format!("sprites/dweller{sprite_i}.png"),
                    },
                    Transform::from_xyz(
                        spawn_pos.x as f32 * TILE_SIZE,
                        spawn_pos.y as f32 * TILE_SIZE,
                        Z_INDEX,
                    ),
                    Pickable::default(),
                ))
                .observe(observe_dweller_hover)
                .observe(despawn_dweller_hover);
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
            Anchor::BOTTOM_CENTER,
        ));
    }
}

pub fn update_dwellers(
    mut commands: Commands,
    mut q_dwellers: Query<(&mut Dweller, &Transform)>,
    tilemap_data: Res<TilemapData>,
    mut q_tasks: Query<(Entity, &mut Task, &TaskNeeds)>,
    mut ev_task_completion: MessageWriter<TaskCompletionEvent>,
    q_mobs: Query<(Entity, &Mob, &Transform)>,
) {
    for (mut dweller, transform) in &mut q_dwellers {
        if !dweller.move_queue.is_empty() {
            continue;
        }

        let pos = transform_to_pos(transform);

        // Check if dweller has a task assigned in all tasks
        let task = q_tasks
            .iter_mut()
            .sort::<&Task>()
            .find(|(_, task, task_needs)| {
                task.dweller_id == Some(dweller.uuid) && dweller.can_do(task.kind, task_needs)
            });

        if let Some((entity_task, mut task, _)) = task {
            if task.reachable_positions.contains(&pos) {
                // Reached task location
                ev_task_completion.write(TaskCompletionEvent { task: entity_task });
            } else {
                // Task moved, try to pathfind again
                if let Some(path) = task.pathfind(pos, &tilemap_data) {
                    debug!("Dweller {} can re-pathfind to {:?}", dweller.name, task);
                    dweller.move_queue = path;
                } else {
                    info!("Dweller {} gives up {:?}", dweller.name, task);
                    task.dweller_id = None;
                }
            }
            continue;
        }

        // Check for nearby hostile mobs to attack
        for (entity_mob, mob, mob_transform) in &q_mobs {
            if !mob.id.data().is_hostile() {
                continue;
            }

            let mob_pos = transform_to_pos(mob_transform);
            let distance_squared = (pos - mob_pos).length_squared();

            if distance_squared <= DWELLER_DETECTION_TILE_RADIUS.pow(2) {
                // New attack task targeting the mob
                let task_entity = commands
                    .spawn(TaskBundle::new_as_child(
                        Task::new(mob_pos, TaskKind::Attack, Some(dweller.uuid)),
                        TaskNeeds::Nothing,
                    ))
                    .id();
                commands.entity(entity_mob).add_child(task_entity);
                break; // Only target one mob at a time
            }
        }

        // Else, wander around
        let mut rng = rand::rng();
        if rng.random_bool(0.2) {
            let directions = tilemap_data.non_blocking_neighbours_pos(pos, true);

            if let Some(direction) = directions.choose(&mut rng) {
                dweller.move_queue.push(*direction);
            }
        }
    }
}

/// Вычисляет utility score задачи для болванчика.
///
/// Формула (по концепту Фазы 2):
/// ```text
/// score = zone_priority * 2.0          ← зональный приоритет (0..10)
///       + (base_priority + 2)           ← тип задачи (-1..2 → 1..4)
///       + 1.0 / distance.max(1.0)      ← близость (0..1)
///       + 0.5                          ← заглушка skill_score (Фаза 3)
/// ```
fn score_task(task_pos: IVec2, dweller_pos: IVec2, zone_priority: u8, task_base_priority: i32) -> f32 {
    let priority_score = zone_priority as f32 * 2.0;
    let base_score = (task_base_priority + 2) as f32; // сдвиг -1..2 → 1..4
    let distance =
        ((dweller_pos.x - task_pos.x).abs() + (dweller_pos.y - task_pos.y).abs()) as f32;
    let distance_score = 1.0 / distance.max(1.0);
    let skill_score = 0.5; // TODO Фаза 3: dweller.skills[task.skill_required]

    priority_score + base_score + distance_score + skill_score
}

pub fn assign_tasks_to_dwellers(
    tilemap_data: Res<TilemapData>,
    zone_map: Res<ZoneMap>,
    mut q_dwellers: Query<(&mut Dweller, &Transform)>,
    mut q_tasks: Query<(Entity, &mut Task, &TaskNeeds)>,
) {
    // Собираем уже занятых болванчиков
    let assigned_dwellers = q_tasks
        .iter()
        .filter_map(|(_, task, _)| task.dweller_id)
        .collect::<HashSet<_>>();

    let mut tasks = q_tasks
        .iter_mut()
        .filter(|(_, task, _)| {
            task.dweller_id.is_none()
                && !task.reachable_positions.is_empty()
                && task.reachable_pathfinding
                // Не назначать задачи, которые заблокируют позицию болванчика
                && (!matches!(task.kind, TaskKind::Build { result } if result.is_blocking())
                    || !q_dwellers
                        .iter()
                        .any(|(_, t)| task.pos == transform_to_pos(t)))
        })
        .collect::<Vec<_>>();

    let mut dwellers = q_dwellers
        .iter_mut()
        .filter_map(|(dweller, transform)| {
            if assigned_dwellers.contains(&dweller.uuid) {
                return None;
            }
            let pos = transform_to_pos(transform);
            Some((dweller, pos))
        })
        .collect::<Vec<_>>();

    // Вычисляем utility score для каждой пары (болванчик, задача)
    let mut pairs: Vec<(f32, usize, usize)> = Vec::new();

    for (dweller_i, (_, dweller_pos)) in dwellers.iter().enumerate() {
        for (task_i, (_, task, _)) in tasks.iter().enumerate() {
            let zone_priority = zone_map.get_priority(task.pos);
            let score = score_task(
                task.pos,
                *dweller_pos,
                zone_priority,
                task.kind.priority(),
            );
            pairs.push((score, dweller_i, task_i));
        }
    }

    // Сортируем по убыванию score (лучшие назначения первыми)
    pairs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut assigned_dwellers = HashSet::new();
    let mut assigned_tasks = HashSet::new();

    for (score, dweller_i, task_i) in pairs {
        if assigned_dwellers.contains(&dweller_i) || assigned_tasks.contains(&task_i) {
            continue;
        }

        let (_, task, task_needs) = &mut tasks[task_i];
        let (dweller, dweller_pos) = &mut dwellers[dweller_i];

        if !dweller.can_do(task.kind, task_needs) {
            continue;
        }

        if let Some(path) = task.pathfind(*dweller_pos, &tilemap_data) {
            task.dweller_id = Some(dweller.uuid);
            dweller.move_queue = path;

            assigned_dwellers.insert(dweller_i);
            assigned_tasks.insert(task_i);
            debug!(
                "Болванчик {} → задача {:?} (score: {:.2})",
                dweller.name, task, score
            );
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

pub fn refresh_pathfinding_tasks_on_mobs(
    tilemap_data: Res<TilemapData>,
    mut q_dwellers: Query<&mut Dweller>,
    mut q_tasks: Query<(&mut Task, &ChildOf)>,
    q_mobs: Query<&Transform, With<Mob>>,
) {
    for (mut task, child_of) in &mut q_tasks {
        if let Ok(mob_transform) = q_mobs.get(child_of.parent()) {
            let mob_pos = transform_to_pos(mob_transform);
            task.pos = mob_pos;
            task.recompute_reachable_positions(&tilemap_data);
            if let Some(mut dweller) = q_dwellers
                .iter_mut()
                .find(|dweller| Some(dweller.uuid) == task.dweller_id)
            {
                // If a dweller is assigned to this task, clear its move queue to force re-pathfinding
                dweller.move_queue.clear();
            }
        }
    }
}
