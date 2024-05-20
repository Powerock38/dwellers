use bevy::prelude::*;
use bevy_entitiles::prelude::*;

use crate::{extract_ok, terrain::TilemapData};

#[derive(Clone, Copy)]
pub struct TileData {
    atlas_index: i32,
    pub wall: bool,
}

impl TileData {
    pub const fn floor(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            wall: false,
        }
    }

    pub const fn wall(atlas_index: i32) -> Self {
        Self {
            atlas_index,
            wall: true,
        }
    }

    pub fn layer(self) -> TileLayer {
        TileLayer::no_flip(self.atlas_index)
    }
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        self.atlas_index == other.atlas_index && self.wall == other.wall
    }
}

pub const GRASS_FLOOR: TileData = TileData::floor(0);
pub const STONE_FLOOR: TileData = TileData::floor(1);
pub const DUNGEON_FLOOR: TileData = TileData::floor(2);
pub const DIRT_WALL: TileData = TileData::wall(4);
pub const STONE_WALL: TileData = TileData::wall(5);
pub const DUNGEON_WALL: TileData = TileData::wall(6);

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
            if tile_data.wall {
                set_tile(
                    *index,
                    STONE_FLOOR,
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
            if tile_data == DIRT_WALL || tile_data == STONE_WALL {
                set_tile(
                    *index,
                    DUNGEON_WALL,
                    &mut commands,
                    &mut tilemap,
                    &mut tilemap_data,
                );

                println!("Smoothened wall at {index:?}");
            } else if tile_data == STONE_FLOOR {
                set_tile(
                    *index,
                    DUNGEON_FLOOR,
                    &mut commands,
                    &mut tilemap,
                    &mut tilemap_data,
                );

                println!("Smoothened floor at {index:?}");
            }
        }
    }
}

pub fn set_tile(
    index: IVec2,
    tile: TileData,
    commands: &mut Commands,
    tilemap: &mut TilemapStorage,
    tilemap_data: &mut TilemapData,
) {
    tilemap.set(
        commands,
        index,
        TileBuilder::new().with_layer(0, tile.layer()),
    );

    tilemap_data.0.set(index, tile);
}
