use bevy::prelude::*;

use crate::{
    data::EAT_VALUES,
    dwellers::Dweller,
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    tilemap_data::TilemapData,
    utils::transform_to_index,
};

const NEEDS_MAX: u32 = 1000;

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
pub struct DwellerNeeds {
    health: u32,
    food: u32,
    sleep: u32,
    cached_speed_ratio: f32,
}

impl Default for DwellerNeeds {
    fn default() -> Self {
        Self {
            health: NEEDS_MAX,
            food: NEEDS_MAX,
            sleep: NEEDS_MAX,
            cached_speed_ratio: 1.0,
        }
    }
}

impl DwellerNeeds {
    pub fn health(&mut self, x: i32) {
        self.health = self.health.saturating_add_signed(x).min(NEEDS_MAX);
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
        let health_ratio = self.health as f32 / NEEDS_MAX as f32;
        let health_speed = 1.0 - (health_ratio - 1.0).abs().powi(2);

        let food_ratio = self.food as f32 / NEEDS_MAX as f32;
        let food_speed = 1.0 - (food_ratio - 1.0).abs().powi(5);

        let sleep_ratio = self.sleep as f32 / NEEDS_MAX as f32;
        let sleep_speed = 1.0 - (sleep_ratio - 1.0).abs().powi(3);

        self.cached_speed_ratio = health_speed.min(food_speed.min(sleep_speed).max(0.1));
    }

    pub fn speed_ratio(&self) -> f32 {
        self.cached_speed_ratio
    }

    pub fn is_fully_rested(&self) -> bool {
        self.sleep == NEEDS_MAX
    }
}

pub fn update_dweller_needs(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut q_needs: Query<(Entity, &mut Dweller, &mut DwellerNeeds, &Transform)>,
    q_tasks: Query<&Task>,
) {
    for (entity, mut dweller, mut needs, transform) in &mut q_needs {
        if needs.health == 0 {
            continue;
        }

        // Dwellers naturally get hungry and tired
        needs.food(-1);
        needs.sleep(-1);

        // If they are not working on something already... (especially an Eat / Sleep task)
        if q_tasks.iter().any(|task| task.dweller == Some(entity)) {
            continue;
        }

        let pos = transform_to_index(transform);

        if needs.food < NEEDS_MAX / 2 {
            if let Some(value) = dweller.object.and_then(|object| EAT_VALUES.get(&object)) {
                needs.food(*value);
                dweller.object = None;
            } else if let Some(pos) = TilemapData::find_from_center_chunk_size(pos, |index| {
                tilemap_data
                    .get(index)
                    .is_some_and(|tile| TaskKind::Eat.is_valid_on_tile(tile))
                    && !q_tasks
                        .iter()
                        .filter(|t| {
                            !(matches!(t.kind, TaskKind::Pickup | TaskKind::Stockpile)
                                && t.dweller.is_none())
                        })
                        .any(|t| t.pos == index)
            }) {
                commands.spawn(TaskBundle::new(
                    Task::new(pos, TaskKind::Eat, Some(entity), &tilemap_data).with_priority(1),
                    TaskNeeds::Nothing,
                ));
            }
        }

        if needs.sleep < NEEDS_MAX / 4 {
            if let Some(pos) = TilemapData::find_from_center_chunk_size(pos, |index| {
                tilemap_data
                    .get(index)
                    .is_some_and(|tile| TaskKind::Sleep.is_valid_on_tile(tile))
                    && !q_tasks.iter().any(|t| t.pos == index)
            }) {
                commands.spawn(TaskBundle::new(
                    Task::new(pos, TaskKind::Sleep, Some(entity), &tilemap_data).with_priority(1),
                    TaskNeeds::Nothing,
                ));
            }
        }
    }
}
