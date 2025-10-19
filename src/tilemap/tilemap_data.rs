use bevy::{platform::collections::HashMap, prelude::*};

use crate::{CHUNK_SIZE, TilePlaced, save_load::SaveName};

pub fn init_tilemap(commands: &mut Commands, save_name: SaveName) {
    commands.insert_resource(save_name);
    commands.insert_resource(TilemapData::default());
}

#[derive(Resource, Default)]
pub struct TilemapData {
    pub chunks: HashMap<IVec2, Vec<TilePlaced>>,
    pub tiles_to_update: HashMap<IVec2, TilePlaced>,
    pub chunks_to_remove: Vec<IVec2>,
}

impl TilemapData {
    pub fn pos_to_chunk_pos_and_local_index(pos: IVec2) -> (IVec2, usize) {
        let size = CHUNK_SIZE as i32;
        let size_vec = IVec2::splat(size);

        let chunk_pos = pos.div_euclid(size_vec);
        let local = pos.rem_euclid(size_vec);

        // flip y: index 0 at top row
        let flipped_y = size - 1 - local.y;
        let index = (flipped_y * size + local.x) as usize;

        (chunk_pos, index)
    }

    pub fn chunk_pos_and_local_index_to_pos(chunk_pos: IVec2, local_index: usize) -> IVec2 {
        let size = CHUNK_SIZE as i32;
        let local_x = (local_index as i32) % size;
        let flipped_y = (local_index as i32) / size;
        let local_y = size - 1 - flipped_y;

        chunk_pos * size + IVec2::new(local_x, local_y)
    }

    pub fn local_pos_to_global(chunk_pos: IVec2, local_pos: IVec2) -> IVec2 {
        chunk_pos * CHUNK_SIZE as i32 + local_pos
    }

    pub fn iter_chunk_positions(chunk_pos: IVec2) -> impl Iterator<Item = IVec2> {
        (0..CHUNK_SIZE).rev().flat_map(move |y| {
            (0..CHUNK_SIZE)
                .map(move |x| Self::local_pos_to_global(chunk_pos, IVec2::new(x as i32, y as i32)))
        })
    }

    pub fn set(&mut self, pos: IVec2, tile: TilePlaced) {
        self.tiles_to_update.insert(pos, tile);
        self.tiles_to_update.extend(self.neighbours(pos)); // necessary for lighting

        let (chunk_pos, tile_index) = Self::pos_to_chunk_pos_and_local_index(pos);
        self.chunks
            .entry(chunk_pos)
            .or_insert_with(|| vec![TilePlaced::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize])
            [tile_index] = tile;
    }

    pub fn get(&self, pos: IVec2) -> Option<TilePlaced> {
        let (chunk_pos, tile_index) = Self::pos_to_chunk_pos_and_local_index(pos);
        self.chunks
            .get(&chunk_pos)
            .and_then(|c| c.get(tile_index))
            .copied()
    }

    pub fn set_chunk(&mut self, chunk_pos: IVec2, chunk_data: Vec<TilePlaced>) {
        self.tiles_to_update.extend(
            chunk_data
                .iter()
                .enumerate()
                .map(|(i, tile)| (Self::chunk_pos_and_local_index_to_pos(chunk_pos, i), *tile)),
        );

        self.chunks.insert(chunk_pos, chunk_data);
    }

    pub fn remove_chunk(&mut self, pos: IVec2) -> Option<Vec<TilePlaced>> {
        self.chunks_to_remove.push(pos);
        self.chunks.remove(&pos)
    }

    pub fn neighbours(&self, pos: IVec2) -> Vec<(IVec2, TilePlaced)> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let neigh_pos = pos + p;

                if let Some(tile) = self.get(neigh_pos) {
                    return Some((neigh_pos, tile));
                }

                None
            })
            .collect()
    }

    pub fn non_blocking_neighbours_pos(&self, pos: IVec2, diagonal: bool) -> Vec<IVec2> {
        let mut result: Vec<IVec2> = self
            .neighbours(pos)
            .into_iter()
            .filter_map(
                |(pos, tile)| {
                    if tile.is_blocking() { None } else { Some(pos) }
                },
            )
            .collect();

        if diagonal {
            const DIAGONAL_DIRECTIONS: [IVec2; 4] = [
                IVec2::new(1, 1),
                IVec2::new(-1, 1),
                IVec2::new(1, -1),
                IVec2::new(-1, -1),
            ];

            for diag in DIAGONAL_DIRECTIONS {
                let diag_pos = pos + diag;

                if let Some(diag_tile) = self.get(diag_pos)
                    && !diag_tile.is_blocking()
                {
                    let adj_blocking = [IVec2::new(diag.x, 0), IVec2::new(0, diag.y)]
                        .into_iter()
                        .any(|adj| {
                            self.get(pos + adj).is_none_or(|t| t.id.data().is_wall())
                            // Do not allow diagonal movement if there is a wall, but allow if it's a blocking object
                        });

                    if !adj_blocking {
                        result.push(diag_pos);
                    }
                }
            }
        }

        result
    }

    pub fn find_from_center_chunk_size(
        center: IVec2,
        is_valid: impl Fn(IVec2) -> bool,
    ) -> Option<IVec2> {
        Self::find_from_center(center, CHUNK_SIZE / 2, is_valid)
    }

    pub fn find_from_center(
        center: IVec2,
        radius: u32,
        is_valid: impl Fn(IVec2) -> bool,
    ) -> Option<IVec2> {
        let radius = radius as i32;

        if is_valid(center) {
            return Some(center);
        }

        // Explore in a spiral pattern
        for layer in 1..=radius {
            let mut position = center + IVec2::new(-layer, -layer);

            // Top edge: Left to right
            for _ in 0..2 * layer {
                if is_valid(position) {
                    return Some(position);
                }
                position.x += 1;
            }

            // Right edge: Top to bottom
            for _ in 0..2 * layer {
                if is_valid(position) {
                    return Some(position);
                }
                position.y += 1;
            }

            // Bottom edge: Right to left
            for _ in 0..2 * layer {
                if is_valid(position) {
                    return Some(position);
                }
                position.x -= 1;
            }

            // Left edge: Bottom to top
            for _ in 0..2 * layer {
                if is_valid(position) {
                    return Some(position);
                }
                position.y -= 1;
            }
        }

        None
    }
}
