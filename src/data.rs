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

enum_map! {
    TileId => TileData {
        GrassFloor = TileData::floor(0),
        StoneFloor = TileData::floor(1),
        DungeonFloor = TileData::floor(2),
        BridgeFloor = TileData::floor(3),
        SandFloor = TileData::floor(4),

        DirtWall = TileData::wall(0),
        StoneWall = TileData::wall(1),
        DungeonWall = TileData::wall(2),
        Water = TileData::wall(3),
    }
}

pub const BUILD_RECIPES: &[(&str, BuildResult, &[ObjectId])] = &[
    (
        "wall",
        BuildResult::Tile(TileId::DungeonWall),
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
