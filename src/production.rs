use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    TilemapData,
    data::{ObjectId, TileId},
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
};

// ─── Константы ────────────────────────────────────────────────────────────────

/// Максимум задач, создаваемых планировщиком за один тик для одного лимита.
const MAX_TASKS_PER_SCHEDULER_RUN: u32 = 8;

/// Максимум записей в логе событий.
const MAX_LOG_ENTRIES: usize = 60;

// ─── Производственные рецепты ──────────────────────────────────────────────────

/// Способ автоматического получения ресурса планировщиком.
#[derive(Debug, Clone)]
pub enum AutoProductionMethod {
    /// Задача Harvest с объектов на тайлах.
    HarvestObject(&'static [ObjectId]),
    /// Задача Dig на тайлах-стенах данного типа.
    DigWall(TileId),
    /// Задача Fish на тайлах с FishingSpot.
    Fish,
    /// Задача Pickup для предметов, уже лежащих на полу.
    PickupLoose(ObjectId),
}

/// Рецепт автоматического производства ресурса.
pub struct AutoRecipe {
    /// Производимый ресурс.
    pub output: ObjectId,
    /// Метод производства.
    pub method: AutoProductionMethod,
}

/// Статическая таблица рецептов автосбора ресурсов.
/// Верстачные цепочки (Bread ← Wheat + Wood) — TODO Фаза 4.
pub static AUTO_RECIPES: &[AutoRecipe] = &[
    AutoRecipe {
        output: ObjectId::Wood,
        method: AutoProductionMethod::HarvestObject(&[ObjectId::Tree, ObjectId::PalmTree]),
    },
    AutoRecipe {
        output: ObjectId::Rock,
        method: AutoProductionMethod::DigWall(TileId::StoneWall),
    },
    AutoRecipe {
        output: ObjectId::Fish,
        method: AutoProductionMethod::Fish,
    },
    AutoRecipe {
        output: ObjectId::Wheat,
        method: AutoProductionMethod::HarvestObject(&[ObjectId::WheatPlant]),
    },
    AutoRecipe {
        output: ObjectId::Berries,
        method: AutoProductionMethod::HarvestObject(&[ObjectId::BerryBush]),
    },
    AutoRecipe {
        output: ObjectId::Honeycomb,
        method: AutoProductionMethod::HarvestObject(&[ObjectId::Beehive]),
    },
    AutoRecipe {
        output: ObjectId::CopperOre,
        method: AutoProductionMethod::PickupLoose(ObjectId::CopperOre),
    },
    AutoRecipe {
        output: ObjectId::Hide,
        method: AutoProductionMethod::PickupLoose(ObjectId::Hide),
    },
    AutoRecipe {
        output: ObjectId::Seeds,
        method: AutoProductionMethod::PickupLoose(ObjectId::Seeds),
    },
];

// ─── Лимиты ресурсов ──────────────────────────────────────────────────────────

/// Лимит запаса одного ресурса.
#[derive(Clone, Debug)]
pub struct ResourceLimit {
    pub resource: ObjectId,
    /// Минимальный запас — ниже этого планировщик создаёт задачи.
    pub min_stock: u32,
    /// Максимальный запас — выше этого планировщик останавливается.
    pub max_stock: u32,
    /// Приоритет автоматически создаваемых задач (1–5).
    pub priority: u8,
    pub enabled: bool,
}

impl ResourceLimit {
    pub fn new(resource: ObjectId, min: u32, max: u32) -> Self {
        Self {
            resource,
            min_stock: min,
            max_stock: max,
            priority: 3,
            enabled: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct ResourceLimits {
    pub limits: Vec<ResourceLimit>,
    /// Видима ли панель лимитов (клавиша L).
    pub panel_visible: bool,
    // UI-состояние формы добавления нового лимита
    pub new_resource_idx: usize,
    pub new_min: u32,
    pub new_max: u32,
}

// ─── Инвентарь ────────────────────────────────────────────────────────────────

/// Текущий подсчёт объектов на карте.
/// Обновляется системой `update_resource_inventory` при изменении тайлмапа.
#[derive(Resource, Default)]
pub struct ResourceInventory {
    pub counts: HashMap<ObjectId, u32>,
}

impl ResourceInventory {
    pub fn get(&self, resource: ObjectId) -> u32 {
        self.counts.get(&resource).copied().unwrap_or(0)
    }
}

// ─── Лог событий ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct LogEntry {
    pub time_secs: f32,
    pub message: String,
}

#[derive(Resource, Default)]
pub struct EventLog {
    pub entries: VecDeque<LogEntry>,
    /// Видим ли лог событий (клавиша J).
    pub visible: bool,
}

impl EventLog {
    pub fn push(&mut self, msg: impl Into<String>, time: f32) {
        if self.entries.len() >= MAX_LOG_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry {
            time_secs: time,
            message: msg.into(),
        });
        debug!("[Лог] {}", self.entries.back().unwrap().message);
    }
}

// ─── Компоненты ───────────────────────────────────────────────────────────────

/// Маркер для задач, созданных планировщиком (для подсчёта активных заданий).
#[derive(Component)]
pub struct SchedulerTask(pub ObjectId);

// ─── Системы ──────────────────────────────────────────────────────────────────

/// Обновляет `ResourceInventory`: подсчитывает все объекты на карте.
/// Запускается только при изменении тайлмапа.
pub fn update_resource_inventory(
    tilemap_data: Res<TilemapData>,
    mut inventory: ResMut<ResourceInventory>,
) {
    if !tilemap_data.is_changed() {
        return;
    }

    inventory.counts.clear();

    for chunk in tilemap_data.chunks.values() {
        for tile in chunk {
            if let Some(object) = tile.object {
                *inventory.counts.entry(object).or_insert(0) += 1;
            }
        }
    }
}

/// Планировщик производства.
///
/// Каждые ~2 секунды проверяет лимиты, сравнивает с запасами
/// и генерирует задачи для болванчиков.
pub fn run_scheduler(
    time: Res<Time>,
    resource_limits: Res<ResourceLimits>,
    inventory: Res<ResourceInventory>,
    mut event_log: ResMut<EventLog>,
    tilemap_data: Res<TilemapData>,
    mut commands: Commands,
    q_tasks: Query<(Entity, &Task, Option<&SchedulerTask>)>,
) {
    let now = time.elapsed_secs();

    for limit in resource_limits.limits.iter().filter(|l| l.enabled) {
        let current = inventory.get(limit.resource);

        if current >= limit.min_stock {
            continue;
        }

        let deficit = limit.min_stock - current;

        // Считаем уже существующие задачи планировщика для этого ресурса
        let existing: u32 = q_tasks
            .iter()
            .filter(|(_, _, sched)| {
                sched.is_some_and(|s| s.0 == limit.resource)
            })
            .count() as u32;

        if existing >= deficit {
            continue;
        }

        let to_create = (deficit - existing).min(MAX_TASKS_PER_SCHEDULER_RUN);

        let Some(recipe) = AUTO_RECIPES.iter().find(|r| r.output == limit.resource) else {
            continue;
        };

        let created = spawn_production_tasks(
            recipe,
            to_create,
            &tilemap_data,
            &q_tasks,
            &mut commands,
        );

        if created > 0 {
            event_log.push(
                format!(
                    "Запас {:?}: {} (мин: {}) → создано {} задач",
                    limit.resource, current, limit.min_stock, created
                ),
                now,
            );
        }
    }
}

/// Создаёт задачи производства по рецепту, возвращает количество созданных.
fn spawn_production_tasks(
    recipe: &AutoRecipe,
    max_tasks: u32,
    tilemap_data: &TilemapData,
    q_tasks: &Query<(Entity, &Task, Option<&SchedulerTask>)>,
    commands: &mut Commands,
) -> u32 {
    let mut created = 0u32;

    match &recipe.method {
        AutoProductionMethod::HarvestObject(source_objects) => {
            'outer: for (chunk_pos_key, chunk) in &tilemap_data.chunks {
                for (i, tile) in chunk.iter().enumerate() {
                    if created >= max_tasks {
                        break 'outer;
                    }

                    let Some(obj) = tile.object else { continue };
                    if !source_objects.contains(&obj) {
                        continue;
                    }

                    let pos = chunk_local_to_global(*chunk_pos_key, i);
                    if task_exists_at(pos, TaskKind::Harvest, q_tasks) {
                        continue;
                    }

                    let needs = if matches!(obj, ObjectId::WheatPlant | ObjectId::BerryBush) {
                        TaskNeeds::EmptyHands
                    } else {
                        TaskNeeds::Nothing
                    };

                    commands
                        .spawn(TaskBundle::new(
                            Task::new(pos, TaskKind::Harvest, None),
                            needs,
                        ))
                        .insert(SchedulerTask(recipe.output));

                    created += 1;
                }
            }
        }

        AutoProductionMethod::DigWall(target_tile) => {
            'outer: for (chunk_pos_key, chunk) in &tilemap_data.chunks {
                for (i, tile) in chunk.iter().enumerate() {
                    if created >= max_tasks {
                        break 'outer;
                    }

                    if tile.id != *target_tile {
                        continue;
                    }

                    let pos = chunk_local_to_global(*chunk_pos_key, i);
                    if task_exists_at(pos, TaskKind::Dig, q_tasks) {
                        continue;
                    }

                    // Нужен хотя бы один проходимый сосед
                    if tilemap_data.non_blocking_neighbours_pos(pos, false).is_empty() {
                        continue;
                    }

                    commands
                        .spawn(TaskBundle::new(
                            Task::new(pos, TaskKind::Dig, None),
                            TaskNeeds::Nothing,
                        ))
                        .insert(SchedulerTask(recipe.output));

                    created += 1;
                }
            }
        }

        AutoProductionMethod::Fish => {
            'outer: for (chunk_pos_key, chunk) in &tilemap_data.chunks {
                for (i, tile) in chunk.iter().enumerate() {
                    if created >= max_tasks {
                        break 'outer;
                    }

                    if !matches!(tile.object, Some(ObjectId::FishingSpot)) {
                        continue;
                    }

                    let pos = chunk_local_to_global(*chunk_pos_key, i);
                    if task_exists_at(pos, TaskKind::Fish, q_tasks) {
                        continue;
                    }

                    commands
                        .spawn(TaskBundle::new(
                            Task::new(pos, TaskKind::Fish, None),
                            TaskNeeds::Nothing,
                        ))
                        .insert(SchedulerTask(recipe.output));

                    created += 1;
                }
            }
        }

        AutoProductionMethod::PickupLoose(loose_object) => {
            'outer: for chunk in tilemap_data.chunks.values() {
                for (i, tile) in chunk.iter().enumerate() {
                    if created >= max_tasks {
                        break 'outer;
                    }

                    if tile.object != Some(*loose_object) {
                        continue;
                    }
                    // Предмет должен лежать на полу, а не быть заблокирован
                    if tile.id.data().is_wall() {
                        continue;
                    }

                    let pos = chunk_index_to_pos(tilemap_data, i);
                    if task_exists_at(pos, TaskKind::Pickup, q_tasks) {
                        continue;
                    }

                    commands
                        .spawn(TaskBundle::new(
                            Task::new(pos, TaskKind::Pickup, None),
                            TaskNeeds::EmptyHands,
                        ))
                        .insert(SchedulerTask(recipe.output));

                    created += 1;
                }
            }
        }
    }

    created
}

// ─── Вспомогательные функции ──────────────────────────────────────────────────

/// Проверяет, существует ли задача данного типа в указанной позиции.
fn task_exists_at(
    pos: IVec2,
    kind: TaskKind,
    q_tasks: &Query<(Entity, &Task, Option<&SchedulerTask>)>,
) -> bool {
    q_tasks.iter().any(|(_, t, _)| t.pos == pos && t.kind == kind)
}

fn chunk_local_to_global(chunk_pos: IVec2, local_index: usize) -> IVec2 {
    use crate::CHUNK_SIZE;
    let size = CHUNK_SIZE as i32;
    let local_x = (local_index as i32) % size;
    let local_y = (local_index as i32) / size;
    chunk_pos * size + IVec2::new(local_x, local_y)
}
