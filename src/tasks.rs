use bevy::prelude::*;
use bevy_entitiles::algorithm::pathfinding::Path;

use crate::dwellers::Dweller;

#[derive(Component)]
pub enum Task {
    Dig(IVec2),
}

pub fn update_pathfinding_tasks(
    mut q_tasks: Query<(&Task, &Parent, &mut Path)>,
    mut q_dwellers: Query<&mut Dweller, With<Children>>,
) {
    for (task, parent, mut path) in &mut q_tasks {
        let Ok(mut dweller) = q_dwellers.get_mut(parent.get()) else {
            continue;
        };

        if dweller.next_move.is_none() {
            dweller.next_move = Some(path.cur_target());
            path.step();
        }
    }
}
