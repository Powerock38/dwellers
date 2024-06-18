use bevy::prelude::*;
use bevy_entitiles::prelude::*;
use bitcode::{Decode, Encode};

use crate::tilemap::{TilemapData, TilemapFiles, TF};

#[derive(Clone, Copy, Encode, Decode, Reflect, Default, Debug)]
pub struct TileData {
    atlas_index: i32,
    pub kind: TileKind,
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        self.atlas_index == other.atlas_index
            && discriminant(&self.kind) == discriminant(&other.kind)
    }
}

impl TileData {
    pub const fn floor(atlas_index: i32) -> Self {
        Self::new(TilemapFiles::FLOORS, atlas_index, TileKind::Floor(None))
    }

    pub const fn wall(atlas_index: i32) -> Self {
        Self::new(TilemapFiles::WALLS, atlas_index, TileKind::Wall)
    }

    const fn new(tf: TF, atlas_index: i32, kind: TileKind) -> Self {
        let atlas_index = TilemapFiles::T.atlas_index(tf, atlas_index);
        Self { atlas_index, kind }
    }

    pub fn with(&self, object_data: ObjectData) -> Self {
        Self {
            atlas_index: self.atlas_index,
            kind: TileKind::Floor(Some(object_data)),
        }
    }

    pub fn without_object(&self) -> Self {
        Self {
            atlas_index: self.atlas_index,
            kind: TileKind::Floor(None),
        }
    }

    pub fn is_blocking(&self) -> bool {
        match self.kind {
            TileKind::Wall => true,
            TileKind::Floor(Some(object_data)) => object_data.blocking,
            _ => false,
        }
    }

    pub fn tile_builder(&self) -> TileBuilder {
        match self.kind {
            TileKind::Floor(Some(object_data)) => TileBuilder::new()
                .with_layer(0, TileLayer::no_flip(self.atlas_index))
                .with_layer(1, TileLayer::no_flip(object_data.atlas_index)),
            _ => TileBuilder::new().with_layer(0, TileLayer::no_flip(self.atlas_index)),
        }
    }

    pub fn set_at(
        &self,
        index: IVec2,
        commands: &mut Commands,
        tilemap: &mut TilemapStorage,
        tilemap_data: &mut TilemapData,
    ) {
        tilemap.set(commands, index, self.tile_builder());

        tilemap_data.set(index, *self);

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
            let tint = if tilemap_data.neighbours(index).iter().all(|(_, tile)| {
                tile == &Self::STONE_WALL || tile == &Self::DIRT_WALL || tile == &Self::DUNGEON_WALL
            }) {
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

#[derive(PartialEq, Clone, Copy, Encode, Decode, Reflect, Default, Debug)]
pub enum TileKind {
    #[default]
    Wall,
    Floor(Option<ObjectData>),
}

#[derive(PartialEq, Clone, Copy, Encode, Decode, Reflect, Default, Debug)]
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
    pub fn carriable(self) -> bool {
        self.carriable
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
