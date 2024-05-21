use bevy::prelude::*;
use bevy_entitiles::prelude::*;

use crate::{extract_ok, terrain::TilemapData};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LayerKind {
    Floor,
    FurnitureNonBlocking,
    FurnitureBlocking,
    Wall,
}

#[derive(Clone, Copy)]
pub struct TileData {
    atlas_index: i32,
    kind: LayerKind,
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
            kind: LayerKind::Floor,
        }
    }

    pub const fn wall(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            kind: LayerKind::Wall,
        }
    }

    pub const fn furniture_non_blocking(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            kind: LayerKind::FurnitureNonBlocking,
        }
    }

    pub const fn furniture_blocking(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            kind: LayerKind::FurnitureBlocking,
        }
    }

    pub fn is_blocking(self) -> bool {
        matches!(self.kind, LayerKind::Wall | LayerKind::FurnitureBlocking)
    }

    pub fn layer(self) -> TileLayer {
        TileLayer::no_flip(self.atlas_index)
    }

    pub fn set_at(
        self,
        index: IVec2,
        commands: &mut Commands,
        tilemap: &mut TilemapStorage,
        tilemap_data: &mut TilemapData,
    ) {
        tilemap.set(
            commands,
            index,
            TileBuilder::new().with_layer(0, self.layer()),
        );

        tilemap_data.0.set(index, self);
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
