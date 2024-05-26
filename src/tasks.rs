use bevy::{prelude::*, sprite::Anchor};

use crate::{
    extract_ok,
    terrain::{TilemapData, TILE_SIZE},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TaskKind {
    Dig,
    Smoothen,
    Chop,
    Bridge,
}

#[derive(Bundle)]
pub struct TaskBundle {
    pub task: Task,
    pub sprite: SpriteBundle,
}

impl TaskBundle {
    pub fn new(task: Task, texture: Handle<Image>) -> Self {
        let x = task.pos.x as f32 * TILE_SIZE;
        let y = task.pos.y as f32 * TILE_SIZE;

        Self {
            task,
            sprite: SpriteBundle {
                texture,
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 1.),
                ..default()
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct Task {
    pub kind: TaskKind,
    pub pos: IVec2,
    pub reachable_positions: Vec<IVec2>,
    pub dweller: Option<Entity>,
}

impl Task {
    pub fn new(pos: IVec2, kind: TaskKind, tilemap_data: &TilemapData) -> Self {
        Self {
            kind,
            pos,
            reachable_positions: Self::compute_reachable_positions(pos, tilemap_data),
            dweller: None,
        }
    }

    pub fn recompute_reachable_positions(&mut self, tilemap_data: &TilemapData) {
        self.reachable_positions = Self::compute_reachable_positions(self.pos, tilemap_data);
    }

    fn compute_reachable_positions(pos: IVec2, tilemap_data: &TilemapData) -> Vec<IVec2> {
        if let Some(tile_data) = tilemap_data.0.get(pos) {
            if !tile_data.is_blocking() {
                return vec![pos];
            }
        }

        tilemap_data.non_blocking_neighbours(pos)
    }
}

pub fn update_unreachable_tasks(
    q_tilemap: Query<&TilemapData, Changed<TilemapData>>,
    mut q_tasks: Query<&mut Task>,
) {
    let tilemap_data = extract_ok!(q_tilemap.get_single());

    for mut task in &mut q_tasks {
        if task.reachable_positions.is_empty() {
            task.recompute_reachable_positions(tilemap_data);
        }
    }
}
