use bevy::prelude::*;
use bevy_entitiles::prelude::*;

use crate::{
    extract_ok,
    terrain::{TilemapData, TilemapFiles, TF},
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FurnitureData {
    atlas_index: i32,
    blocking: bool,
}

impl FurnitureData {
    pub const TABLE: Self = Self::blocking(0);
    pub const RUG: Self = Self::passable(1);

    pub const fn passable(atlas_index: i32) -> Self {
        Self::new(atlas_index, false)
    }

    pub const fn blocking(atlas_index: i32) -> Self {
        Self::new(atlas_index, true)
    }

    const fn new(atlas_index: i32, blocking: bool) -> Self {
        let atlas_index = TilemapFiles::T.atlas_index(TilemapFiles::FURNITURES, atlas_index);
        Self {
            atlas_index,
            blocking,
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

pub enum TileEvent {
    Dig,
    Smoothen,
    Chop,
    Bridge,
}

#[derive(Event)]
pub struct SetTileEvent {
    index: IVec2,
    event: TileEvent,
}

impl SetTileEvent {
    pub fn new(index: IVec2, event: TileEvent) -> Self {
        Self { index, event }
    }
}

pub fn event_set_tile(
    mut commands: Commands,
    mut events: EventReader<SetTileEvent>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    for event in events.read() {
        if let Some(tile_data) = tilemap_data.0.get(event.index) {
            match event.event {
                TileEvent::Dig => {
                    if tile_data.is_blocking() {
                        TileData::STONE_FLOOR.set_at(
                            event.index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        println!("Dug tile at {:?}", event.index);
                    }
                }

                TileEvent::Smoothen => {
                    if tile_data == TileData::DIRT_WALL || tile_data == TileData::STONE_WALL {
                        TileData::DUNGEON_WALL.set_at(
                            event.index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        println!("Smoothened wall at {:?}", event.index);
                    } else if tile_data == TileData::STONE_FLOOR {
                        TileData::DUNGEON_FLOOR.set_at(
                            event.index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        println!("Smoothened floor at {:?}", event.index);
                    }
                }

                TileEvent::Chop => {
                    if tile_data == TileData::TREE {
                        TileData::GRASS_FLOOR.set_at(
                            event.index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        println!("Chopped tile at {:?}", event.index);
                    }
                }

                TileEvent::Bridge => {
                    if tile_data == TileData::WATER {
                        TileData::BRIDGE_FLOOR.set_at(
                            event.index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        println!("Bridged tile at {:?}", event.index);
                    }
                }
            }
        }
    }
}
