use bevy::prelude::*;

use crate::{
    TilemapData,
    data::EAT_VALUES,
    dwellers::{Dweller, NEEDS_MAX},
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    utils::transform_to_pos,
};

pub fn update_dweller_needs(
    mut commands: Commands,
    tilemap_data: Res<TilemapData>,
    mut q_needs: Query<(&mut Dweller, &Transform)>,
    q_tasks: Query<&Task>,
) {
    for (mut dweller, transform) in &mut q_needs {
        if dweller.health == 0 {
            continue;
        }

        // Dwellers naturally get hungry and tired
        dweller.food(-1);
        dweller.sleep(-1);

        // If they are not working on something already... (especially an Eat / Sleep task)
        if q_tasks
            .iter()
            .any(|task| task.dweller_id == Some(dweller.uuid))
        {
            continue;
        }

        let pos = transform_to_pos(transform);

        if dweller.food < NEEDS_MAX / 2 {
            if let Some(value) = dweller.object.and_then(|object| EAT_VALUES.get(&object)) {
                dweller.food(*value);
                dweller.object = None;
            } else if let Some(pos) = TilemapData::find_from_center_chunk_size(pos, |pos| {
                tilemap_data
                    .get(pos)
                    .is_some_and(|tile| TaskKind::Eat.is_valid_on_tile(tile))
                    && !q_tasks
                        .iter()
                        .filter(|t| {
                            !(matches!(t.kind, TaskKind::Pickup | TaskKind::Stockpile)
                                && t.dweller_id.is_none())
                        })
                        .any(|t| t.pos == pos)
            }) {
                commands.spawn(TaskBundle::new(
                    Task::new(pos, TaskKind::Eat, Some(dweller.uuid)),
                    TaskNeeds::Nothing,
                ));
            }
        }

        if dweller.sleep < NEEDS_MAX / 4
            && let Some(pos) = TilemapData::find_from_center_chunk_size(pos, |pos| {
                tilemap_data
                    .get(pos)
                    .is_some_and(|tile| TaskKind::Sleep.is_valid_on_tile(tile))
                    && !q_tasks.iter().any(|t| t.pos == pos)
            })
        {
            commands.spawn(TaskBundle::new(
                Task::new(pos, TaskKind::Sleep, Some(dweller.uuid)),
                TaskNeeds::Nothing,
            ));
        }
    }
}
