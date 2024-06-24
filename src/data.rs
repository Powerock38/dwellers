use std::sync::LazyLock;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bitcode::{Decode, Encode};

use crate::{enum_map, BuildResult, ObjectData, TileData};

enum_map! {
    ObjectId => ObjectData {
        Wood = ObjectData::passable(0),
        Rug = ObjectData::passable(1),
        Tree = ObjectData::blocking_non_carriable(2),
        Table = ObjectData::blocking(3),
        Stool = ObjectData::blocking(4),
        Bed = ObjectData::blocking(5),
        Door = ObjectData::passable(6),
        Rock = ObjectData::passable(7),
        TallGrass = ObjectData::passable_non_carriable(8),
        Seeds = ObjectData::passable(9),
        Farm = ObjectData::passable_non_carriable(10),
        WheatPlant = ObjectData::passable_non_carriable(11),
        Wheat = ObjectData::passable(12),
        Furnace = ObjectData::blocking(13),
        Bread = ObjectData::passable(14),
        PalmTree = ObjectData::blocking_non_carriable(15),
        Cactus = ObjectData::passable_non_carriable(16),
        CopperOre = ObjectData::passable(17),
        CopperIngot = ObjectData::passable(18),
    }
}

//TODO: make enum like ObjectData and allow walls to contain objects (ores) but keep pickup logic for floor only
impl TileData {
    pub const GRASS_FLOOR: Self = Self::floor(0);
    pub const STONE_FLOOR: Self = Self::floor(1);
    pub const DUNGEON_FLOOR: Self = Self::floor(2);
    pub const BRIDGE_FLOOR: Self = Self::floor(3);
    pub const SAND_FLOOR: Self = Self::floor(4);

    pub const DIRT_WALL: Self = Self::wall(0);
    pub const STONE_WALL: Self = Self::wall(1);
    pub const DUNGEON_WALL: Self = Self::wall(2);
    pub const WATER: Self = Self::wall(3);
}

pub const BUILD_RECIPES: &[(&str, BuildResult, &[ObjectId])] = &[
    (
        "wall",
        BuildResult::Tile(TileData::DUNGEON_WALL),
        &[ObjectId::Rock],
    ),
    (
        "table",
        BuildResult::Object(ObjectId::Table),
        &[ObjectId::Wood, ObjectId::Wood],
    ),
    (
        "stool",
        BuildResult::Object(ObjectId::Stool),
        &[ObjectId::Wood],
    ),
    ("bed", BuildResult::Object(ObjectId::Bed), &[ObjectId::Wood]),
    (
        "door",
        BuildResult::Object(ObjectId::Door),
        &[ObjectId::Wood],
    ),
    (
        "farm",
        BuildResult::Object(ObjectId::Farm),
        &[ObjectId::Seeds],
    ),
    (
        "furnace",
        BuildResult::Object(ObjectId::Furnace),
        &[ObjectId::Rock, ObjectId::Rock, ObjectId::Rock],
    ),
];

pub static WORKSTATIONS: LazyLock<HashMap<ObjectId, (ObjectId, Vec<ObjectId>)>> =
    LazyLock::new(|| {
        HashMap::from([(
            ObjectId::Furnace,
            (ObjectId::Bread, vec![ObjectId::Wheat, ObjectId::Wood]),
        )])
    });
