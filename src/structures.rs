use bevy::math::IVec2;

use crate::{data::MobId, TilePlaced};

pub struct StructureData {
    tiles: Vec<Vec<TilePlaced>>,
    mobs: Vec<(IVec2, MobId)>,
}

impl StructureData {
    pub fn new(mut tiles: Vec<Vec<TilePlaced>>, mobs: Vec<(IVec2, MobId)>) -> Self {
        tiles.reverse();
        StructureData { tiles, mobs }
    }

    pub fn x_size(&self) -> usize {
        self.tiles.iter().map(Vec::len).max().unwrap_or(0)
    }

    pub fn y_size(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&TilePlaced> {
        self.tiles.get(y)?.get(x)
    }

    pub fn mobs(&self) -> &[(IVec2, MobId)] {
        &self.mobs
    }
}
