use bevy::math::UVec2;

use crate::{data::MobId, TilePlaced};

pub struct StructureData {
    tiles: Vec<Vec<Option<TilePlaced>>>,
    mobs: Vec<(UVec2, MobId)>,
}

impl StructureData {
    pub fn new(tiles: Vec<Vec<Option<TilePlaced>>>, mobs: Vec<(u32, u32, MobId)>) -> Self {
        StructureData {
            tiles,
            mobs: mobs
                .into_iter()
                .map(|(x, y, m)| (UVec2::new(x, y), m))
                .collect(),
        }
    }

    pub fn x_size(&self) -> usize {
        self.tiles.iter().map(Vec::len).max().unwrap_or_default()
    }

    pub fn y_size(&self) -> usize {
        self.tiles.len()
    }

    pub fn size(&self) -> UVec2 {
        UVec2::new(self.x_size() as u32, self.y_size() as u32)
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&TilePlaced> {
        self.tiles.get(y)?.get(x)?.as_ref()
    }

    pub fn mobs(&self) -> &[(UVec2, MobId)] {
        &self.mobs
    }

    pub fn flip_vertical(&self) -> Self {
        let mut tiles = self.tiles.clone();
        tiles.reverse();
        StructureData {
            tiles,
            mobs: self
                .mobs
                .iter()
                .map(|(pos, mob)| (UVec2::new(pos.x, self.y_size() as u32 - 1 - pos.y), *mob))
                .collect(),
        }
    }

    pub fn flip_horizontal(&self) -> Self {
        let mut tiles = self.tiles.clone();
        let x_size = self.x_size();
        for row in &mut tiles {
            row.resize(x_size, None);
            row.reverse();
            // trim None at the end
            while matches!(row.last(), Some(None)) {
                row.pop();
            }
        }
        StructureData {
            tiles,
            mobs: self
                .mobs
                .iter()
                .map(|(pos, mob)| (UVec2::new(self.x_size() as u32 - 1 - pos.x, pos.y), *mob))
                .collect(),
        }
    }

    fn rotate_pos(&self, pos: UVec2, clockwise: bool) -> UVec2 {
        if clockwise {
            UVec2::new(pos.y, self.x_size() as u32 - 1 - pos.x)
        } else {
            UVec2::new(self.y_size() as u32 - 1 - pos.y, pos.x)
        }
    }

    pub fn rotate(&self, clockwise: bool) -> Self {
        let mut tiles = vec![vec![None; self.y_size()]; self.x_size()];

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let pos = self.rotate_pos(UVec2::new(x as u32, y as u32), clockwise);
                tiles[pos.y as usize][pos.x as usize] = *tile;
            }
        }

        StructureData {
            tiles,
            mobs: self
                .mobs
                .iter()
                .map(|(pos, mob)| (self.rotate_pos(*pos, clockwise), *mob))
                .collect(),
        }
    }
}
