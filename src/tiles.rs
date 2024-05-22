use bevy::prelude::*;
use bevy_entitiles::prelude::*;

use crate::{extract_ok, terrain::TilemapData};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FurnitureData {
    atlas_index: i32,
    blocking: bool,
}

impl FurnitureData {
    pub const TABLE: Self = Self::blocking(3);
    pub const RUG: Self = Self::passable(7);

    pub const fn passable(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            blocking: false,
        }
    }

    pub const fn blocking(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            blocking: true,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TileKind {
    Floor(Option<FurnitureData>),
    Wall,
}

#[derive(Clone, Copy)]
pub struct TileData {
    atlas_index: i32,
    kind: TileKind,
}

impl TileData {
    pub const GRASS_FLOOR: Self = Self::floor(0);
    pub const STONE_FLOOR: Self = Self::floor(1);
    pub const DUNGEON_FLOOR: Self = Self::floor(2);
    pub const DIRT_WALL: Self = Self::wall(4);
    pub const STONE_WALL: Self = Self::wall(5);
    pub const DUNGEON_WALL: Self = Self::wall(6);

    pub const fn floor(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            kind: TileKind::Floor(None),
        }
    }

    pub const fn wall(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            kind: TileKind::Wall,
        }
    }

    pub fn is_blocking(&self) -> bool {
        self.kind == TileKind::Wall
    }

    pub fn tile_builder(&self) -> TileBuilder {
        match self.kind {
            TileKind::Floor(Some(furniture_data)) => TileBuilder::new()
                .with_layer(0, TileLayer::no_flip(self.atlas_index))
                .with_layer(1, TileLayer::no_flip(furniture_data.atlas_index)),
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
    }
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        self.atlas_index == other.atlas_index && self.kind == other.kind
    }
}

#[derive(Event)]
pub struct MineTile(pub IVec2);

pub fn event_mine_tile(
    mut commands: Commands,
    mut ev_mine_tile: EventReader<MineTile>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    for MineTile(index) in ev_mine_tile.read() {
        if let Some(tile_data) = tilemap_data.0.get(*index) {
            if tile_data.is_blocking() {
                TileData::STONE_FLOOR.set_at(
                    *index,
                    &mut commands,
                    &mut tilemap,
                    &mut tilemap_data,
                );

                println!("Mined tile at {index:?}");
            }
        }
    }
}

#[derive(Event)]
pub struct SmoothenTile(pub IVec2);

pub fn event_smoothen_tile(
    mut commands: Commands,
    mut ev_smoothen_tile: EventReader<SmoothenTile>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    for SmoothenTile(index) in ev_smoothen_tile.read() {
        if let Some(tile_data) = tilemap_data.0.get(*index) {
            if tile_data == TileData::DIRT_WALL || tile_data == TileData::STONE_WALL {
                TileData::DUNGEON_WALL.set_at(
                    *index,
                    &mut commands,
                    &mut tilemap,
                    &mut tilemap_data,
                );

                println!("Smoothened wall at {index:?}");
            } else if tile_data == TileData::STONE_FLOOR {
                TileData::DUNGEON_FLOOR.set_at(
                    *index,
                    &mut commands,
                    &mut tilemap,
                    &mut tilemap_data,
                );

                println!("Smoothened floor at {index:?}");
            }
        }
    }
}
