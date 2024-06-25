use bevy::prelude::*;
use bevy_entitiles::prelude::*;
use bitcode::{Decode, Encode};

use crate::{
    data::{ObjectId, TileId},
    tilemap::{TilemapData, TilemapFiles, TF},
};

#[derive(Clone, Copy, Encode, Decode, Reflect, Default, Debug)]
pub struct TilePlaced {
    pub id: TileId,
    pub object: Option<ObjectId>,
}

impl TilePlaced {
    pub fn is_blocking(self) -> bool {
        TileId::data(&self.id).wall
            || self
                .object
                .map_or(false, |o| ObjectId::data(&o).is_blocking())
    }

    pub fn is_floor_free(self) -> bool {
        !self.is_blocking() && self.object.is_none()
    }

    pub fn tile_builder(self) -> TileBuilder {
        let mut tb =
            TileBuilder::new().with_layer(0, TileLayer::no_flip(self.id.data().atlas_index));

        if let Some(object) = self.object {
            tb = tb.with_layer(1, TileLayer::no_flip(object.data().atlas_index));
        }

        tb
    }

    pub fn set_at(
        self,
        index: IVec2,
        commands: &mut Commands,
        tilemap: &mut TilemapStorage,
        tilemap_data: &mut TilemapData,
    ) {
        tilemap.set(commands, index, self.tile_builder());

        tilemap_data.set(index, self);

        Self::update_light_level(index, commands, tilemap, tilemap_data);
    }

    pub fn update_light_level(
        index: IVec2,
        commands: &mut Commands,
        tilemap: &mut TilemapStorage,
        tilemap_data: &TilemapData,
    ) {
        for index in tilemap_data
            .neighbours(index)
            .iter()
            .map(|n| n.0)
            .chain([index])
        {
            let tint =
                if tilemap_data
                    .neighbours(index)
                    .iter()
                    .all(|(_, TilePlaced { id: tile, .. })| {
                        *tile == TileId::StoneWall
                            || *tile == TileId::DirtWall
                            || *tile == TileId::DungeonWall
                    })
                {
                    Color::BLACK
                } else {
                    Color::WHITE
                };

            tilemap.update(
                commands,
                index,
                TileUpdater {
                    tint: Some(tint),
                    ..default()
                },
            );
        }
    }
}

pub struct TileData {
    atlas_index: i32,
    wall: bool,
}

impl TileData {
    pub const fn floor(atlas_index: i32) -> Self {
        Self::new(TilemapFiles::FLOORS, atlas_index, false)
    }

    pub const fn wall(atlas_index: i32) -> Self {
        Self::new(TilemapFiles::WALLS, atlas_index, true)
    }

    pub const fn new(tf: TF, atlas_index: i32, wall: bool) -> Self {
        let atlas_index = TilemapFiles::T.atlas_index(tf, atlas_index);
        Self { atlas_index, wall }
    }

    #[inline]
    pub fn is_wall(&self) -> bool {
        self.wall
    }
}

impl TileId {
    pub fn with(self, object_id: ObjectId) -> TilePlaced {
        TilePlaced {
            id: self,
            object: Some(object_id),
        }
    }

    pub fn empty(self) -> TilePlaced {
        TilePlaced {
            id: self,
            object: None,
        }
    }
}

pub struct ObjectData {
    atlas_index: i32,
    blocking: bool,
    carriable: bool,
}

impl ObjectData {
    pub const fn passable(atlas_index: i32) -> Self {
        Self::new(atlas_index, false, true)
    }

    pub const fn blocking(atlas_index: i32) -> Self {
        Self::new(atlas_index, true, true)
    }

    pub const fn passable_non_carriable(atlas_index: i32) -> Self {
        Self::new(atlas_index, false, false)
    }

    pub const fn blocking_non_carriable(atlas_index: i32) -> Self {
        Self::new(atlas_index, true, false)
    }

    #[inline]
    pub fn is_carriable(&self) -> bool {
        self.carriable
    }

    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    const fn new(atlas_index: i32, blocking: bool, carriable: bool) -> Self {
        let atlas_index = TilemapFiles::T.atlas_index(TilemapFiles::OBJECTS, atlas_index);
        Self {
            atlas_index,
            blocking,
            carriable,
        }
    }
}
