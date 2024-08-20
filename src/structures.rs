use bevy::math::IVec2;

use crate::{data::MobId, TilePlaced};

pub struct StructureData {
    tiles: Vec<Vec<Option<TilePlaced>>>,
    mobs: Vec<(IVec2, MobId)>,
}

impl StructureData {
    pub fn new(mut tiles: Vec<Vec<Option<TilePlaced>>>, mobs: Vec<((u32, u32), MobId)>) -> Self {
        tiles.reverse();
        StructureData {
            tiles,
            mobs: mobs
                .into_iter()
                .map(|((x, y), m)| (IVec2::new(x as i32, y as i32), m))
                .collect(),
        }
    }

    pub fn x_size(&self) -> usize {
        self.tiles.iter().map(Vec::len).max().unwrap_or(0)
    }

    pub fn y_size(&self) -> usize {
        self.tiles.len()
    }

    pub fn size(&self) -> IVec2 {
        IVec2::new(self.x_size() as i32, self.y_size() as i32)
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&TilePlaced> {
        self.tiles.get(y)?.get(x)?.as_ref()
    }

    pub fn mobs(&self) -> &[(IVec2, MobId)] {
        &self.mobs
    }
}
