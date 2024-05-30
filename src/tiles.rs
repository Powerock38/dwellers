use bevy::prelude::*;
use bevy_entitiles::prelude::*;

use crate::terrain::{TilemapData, TilemapFiles, TF};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ObjectData {
    atlas_index: i32,
    blocking: bool,
}

impl ObjectData {
    pub const WOOD: Self = Self::blocking(0);
    pub const TABLE: Self = Self::blocking(1);
    pub const RUG: Self = Self::passable(2);

    pub const fn passable(atlas_index: i32) -> Self {
        Self::new(atlas_index, false)
    }

    pub const fn blocking(atlas_index: i32) -> Self {
        Self::new(atlas_index, true)
    }

    const fn new(atlas_index: i32, blocking: bool) -> Self {
        let atlas_index = TilemapFiles::T.atlas_index(TilemapFiles::OBJECTS, atlas_index);
        Self {
            atlas_index,
            blocking,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum TileKind {
    Floor(Option<ObjectData>),
    Wall,
}

#[derive(Clone, Copy)]
pub struct TileData {
    atlas_index: i32,
    pub kind: TileKind,
}

impl TileData {
    pub const GRASS_FLOOR: Self = Self::floor(0);
    pub const STONE_FLOOR: Self = Self::floor(1);
    pub const DUNGEON_FLOOR: Self = Self::floor(2);
    pub const BRIDGE_FLOOR: Self = Self::floor(3);
    pub const DIRT_WALL: Self = Self::wall(0);
    pub const STONE_WALL: Self = Self::wall(1);
    pub const DUNGEON_WALL: Self = Self::wall(2);
    pub const TREE: Self = Self::wall(3);
    pub const WATER: Self = Self::wall(4);

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

    pub fn is_blocking(&self) -> bool {
        self.kind == TileKind::Wall
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

        tilemap_data.0.set(index, *self);

        Self::update_light_level(index, commands, tilemap, tilemap_data);
        for (neighbour_index, _) in tilemap_data.neighbours(index) {
            Self::update_light_level(neighbour_index, commands, tilemap, tilemap_data);
        }
    }

    pub fn update_light_level(
        index: IVec2,
        commands: &mut Commands,
        tilemap: &mut TilemapStorage,
        tilemap_data: &mut TilemapData,
    ) {
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

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        self.atlas_index == other.atlas_index
            && discriminant(&self.kind) == discriminant(&other.kind)
    }
}
