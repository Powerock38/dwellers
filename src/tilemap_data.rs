use bevy::{prelude::*, utils::HashMap};

use crate::{utils::div_to_floor, TilePlaced, CHUNK_SIZE};

#[derive(Resource, Default)]
pub struct TilemapData {
    pub chunks: HashMap<IVec2, Vec<Option<TilePlaced>>>,
    pub tiles_to_update: HashMap<IVec2, TilePlaced>,
    pub chunks_to_remove: Vec<IVec2>,
}

impl TilemapData {
    #[inline]
    pub fn index_to_chunk(index: IVec2) -> (IVec2, usize) {
        let isize = IVec2::splat(CHUNK_SIZE as i32);
        let c = div_to_floor(index, isize);
        let idx = index - c * isize;
        (c, (idx.y * isize.x + idx.x) as usize)
    }

    #[inline]
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
            .or_insert_with(|| vec![None; (CHUNK_SIZE * CHUNK_SIZE) as usize])[idx.1] = Some(tile);
    }

    pub fn get(&self, index: IVec2) -> Option<TilePlaced> {
        let idx = Self::index_to_chunk(index);
        self.chunks
            .get(&idx.0)
            .and_then(|c| c.get(idx.1))
            .copied()
            .flatten()
    }

    pub fn set_chunk(&mut self, chunk_index: IVec2, chunk_data: Vec<TilePlaced>) {
        self.tiles_to_update.extend(
            chunk_data
                .iter()
                .enumerate()
                .map(|(i, tile)| (Self::chunk_to_index(chunk_index, i), *tile)),
        );

        self.chunks
            .insert(chunk_index, chunk_data.into_iter().map(Some).collect());
    }

    pub fn remove_chunk(&mut self, index: IVec2) -> Option<Vec<Option<TilePlaced>>> {
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
                                self.get(pos + adj).map_or(true, |t| t.id.data().is_wall())
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

    pub fn find_from_center(index: IVec2, is_valid: impl Fn(IVec2) -> bool) -> Option<IVec2> {
        for radius in 0..=(CHUNK_SIZE as i32 / 2) {
            // Top and Bottom edges of the square
            for x in (index.x - radius).max(1)..=(index.x + radius).min(CHUNK_SIZE as i32 - 2) {
                // Top edge
                let top_y = (index.y - radius).max(1);
                if is_valid(IVec2::new(x, top_y)) {
                    return Some(IVec2::new(x, top_y));
                }
                // Bottom edge
                let bottom_y = (index.y + radius).min(CHUNK_SIZE as i32 - 2);
                if is_valid(IVec2::new(x, bottom_y)) {
                    return Some(IVec2::new(x, bottom_y));
                }
            }

            // Left and Right edges of the square (excluding corners already checked)
            for y in ((index.y - radius + 1).max(1))
                ..=((index.y + radius - 1).min(CHUNK_SIZE as i32 - 2))
            {
                // Left edge
                let left_x = (index.x - radius).max(1);
                if is_valid(IVec2::new(left_x, y)) {
                    return Some(IVec2::new(left_x, y));
                }
                // Right edge
                let right_x = (index.x + radius).min(CHUNK_SIZE as i32 - 2);
                if is_valid(IVec2::new(right_x, y)) {
                    return Some(IVec2::new(right_x, y));
                }
            }
        }

        None
    }
}
