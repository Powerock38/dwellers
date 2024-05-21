use bevy::prelude::*;
use bevy_entitiles::{
    algorithm::pathfinding::PathTilemaps,
    math::extension::TileIndex,
    prelude::*,
    tilemap::algorithm::path::{PathTile, PathTilemap},
};

use crate::{
    extract_ok,
    terrain::{TilemapData, TERRAIN_SIZE},
};

#[derive(Debug)]
pub enum TaskKind {
    Dig,
    Smoothen,
}

#[derive(Component, Debug)]
pub struct Task {
    pub kind: TaskKind,
    pub pos: IVec2,
    pub pos_adjacent: Vec<IVec2>,
}

impl Task {
    pub fn new(pos: IVec2, kind: TaskKind, tilemap_data: &TilemapData) -> Self {
        Self {
            kind,
            pos,
            pos_adjacent: Self::compute_pos_adjacent(pos, tilemap_data),
        }
    }

    pub fn recompute_pos_adjacent(&mut self, tilemap_data: &TilemapData) {
        self.pos_adjacent = Self::compute_pos_adjacent(self.pos, tilemap_data);
    }

    fn compute_pos_adjacent(pos: IVec2, tilemap_data: &TilemapData) -> Vec<IVec2> {
        pos.neighbours(TilemapType::Square, false)
            .into_iter()
            .filter_map(|pos| {
                if let Some(pos) = pos {
                    if let Some(tile_data) = tilemap_data.0.get(pos) {
                        if !tile_data.is_blocking() {
                            return Some(pos);
                        }
                    }
                }
                None
            })
            .collect()
    }
}

pub fn update_path_tilemaps(
    q_tilemap: Query<(Entity, &TilemapData), Changed<TilemapData>>,
    mut q_tasks: Query<&mut Task>,
    mut path_tilemaps: ResMut<PathTilemaps>,
) {
    let (entity, tilemap_data) = extract_ok!(q_tilemap.get_single());

    // Update pathfinding tilemap
    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        |index| {
            if let Some(tile_data) = tilemap_data.0.get(index) {
                if !tile_data.is_blocking() {
                    return Some(PathTile { cost: 1 });
                }
            }

            Some(PathTile { cost: u32::MAX / 2 })
        },
    );
    path_tilemaps.insert(entity, path_tilemap);

    // Update unreachable Tasks
    for mut task in &mut q_tasks {
        if task.pos_adjacent.is_empty() {
            task.recompute_pos_adjacent(tilemap_data);
        }
    }
}
