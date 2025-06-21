use bevy::{platform::collections::HashMap, prelude::*};

use crate::{utils::div_to_floor, TilePlaced, CHUNK_SIZE};

#[derive(Resource, Default)]
pub struct TilemapData {
    pub chunks: HashMap<IVec2, Vec<TilePlaced>>,
    pub tiles_to_update: HashMap<IVec2, TilePlaced>,
    pub chunks_to_remove: Vec<IVec2>,
}

impl TilemapData {
    pub fn index_to_chunk(index: IVec2) -> (IVec2, usize) {
        let isize = IVec2::splat(CHUNK_SIZE as i32);
        let chunk_index = div_to_floor(index, isize);
        let idx = index - chunk_index * isize;
        (chunk_index, (idx.y * isize.x + idx.x) as usize)
    }

    pub fn chunk_to_index(chunk_index: IVec2, local_index: usize) -> IVec2 {
        chunk_index * CHUNK_SIZE as i32
            + IVec2::new(
                local_index as i32 % CHUNK_SIZE as i32,
                local_index as i32 / CHUNK_SIZE as i32,
            )
    }

    pub fn set(&mut self, index: IVec2, tile: TilePlaced) {
        self.tiles_to_update.insert(index, tile);
        self.tiles_to_update.extend(self.neighbours(index)); // necessary for lighting

        let idx = Self::index_to_chunk(index);
        self.chunks
            .entry(idx.0)
            .or_insert_with(|| vec![TilePlaced::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize])
            [idx.1] = tile;
    }

    pub fn get(&self, index: IVec2) -> Option<TilePlaced> {
        let idx = Self::index_to_chunk(index);
        self.chunks.get(&idx.0).and_then(|c| c.get(idx.1)).copied()
    }

    pub fn set_chunk(&mut self, chunk_index: IVec2, chunk_data: Vec<TilePlaced>) {
        self.tiles_to_update.extend(
            chunk_data
                .iter()
                .enumerate()
                .map(|(i, tile)| (Self::chunk_to_index(chunk_index, i), *tile)),
        );

        self.chunks.insert(chunk_index, chunk_data);
    }

    pub fn remove_chunk(&mut self, index: IVec2) -> Option<Vec<TilePlaced>> {
        self.chunks_to_remove.push(index);
        self.chunks.remove(&index)
    }

    pub fn local_index_to_global(chunk_index: IVec2, local_index: IVec2) -> IVec2 {
        chunk_index * CHUNK_SIZE as i32 + local_index
    }

    pub fn neighbours(&self, pos: IVec2) -> Vec<(IVec2, TilePlaced)> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let index = pos + p;

                if let Some(tile) = self.get(index) {
                    return Some((index, tile));
                }

                None
            })
            .collect()
    }

    pub fn non_blocking_neighbours_pos(&self, pos: IVec2, diagonal: bool) -> Vec<IVec2> {
        let mut result: Vec<IVec2> = self
            .neighbours(pos)
            .into_iter()
            .filter_map(|(index, tile)| {
                if tile.is_blocking() {
                    None
                } else {
                    Some(index)
                }
            })
            .collect();

        if diagonal {
            let diagonal_directions = [
                IVec2::new(1, 1),
                IVec2::new(-1, 1),
                IVec2::new(1, -1),
                IVec2::new(-1, -1),
            ];

            for diag_pos in diagonal_directions {
                let diag_index = pos + diag_pos;

                if let Some(diag_tile) = self.get(diag_index) {
                    if !diag_tile.is_blocking() {
                        let adj_blocking = [IVec2::new(diag_pos.x, 0), IVec2::new(0, diag_pos.y)]
                            .into_iter()
                            .any(|adj| {
                                self.get(pos + adj).is_none_or(|t| t.id.data().is_wall())
                                // Do not allow diagonal movement if there is a wall, but allow if it's a blocking object
                            });

                        if !adj_blocking {
                            result.push(diag_index);
                        }
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
