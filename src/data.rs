use std::sync::LazyLock;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bitcode::{Decode, Encode};

use crate::{enum_map, structures::StructureData, BuildResult, MobData, ObjectData, TileData};

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
        WoodWall = TileData::wall(4),
    }
}

enum_map! {
    MobId => MobData {
        Sheep = MobData::new("sheep", 60.0, ObjectId::Rug),
        Boar = MobData::new("boar", 50.0, ObjectId::Rug),
        Undead = MobData::new("undead", 40.0, ObjectId::CopperIngot),
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

enum_map! {
    StructureId => StructureData {
        SmallHouse = StructureData::new(vec![
                vec![TileId::DungeonWall.s(), TileId::DungeonWall.s(),               TileId::DungeonWall.s(),              TileId::DungeonWall.s()],
                vec![TileId::DungeonWall.s(), TileId::DungeonFloor.s(),              TileId::DungeonFloor.i(ObjectId::Bed), TileId::DungeonWall.s()],
                vec![TileId::DungeonWall.s(), TileId::DungeonFloor.s(),              TileId::DungeonFloor.s(),             TileId::DungeonWall.s()],
                vec![TileId::DungeonWall.s(), TileId::DungeonFloor.i(ObjectId::Door), TileId::DungeonWall.s(),              TileId::DungeonWall.s()],
            ],
            vec![((1,1), MobId::Sheep)],
        ),

        SmallOutpost = StructureData::new(vec![
                vec![None,                 None,                     TileId::WoodWall.s(),     TileId::WoodWall.s(),     TileId::WoodWall.s(),                       TileId::WoodWall.s(),     TileId::WoodWall.s()],
                vec![None,                 TileId::WoodWall.s(),     TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![TileId::WoodWall.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![TileId::WoodWall.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![TileId::WoodWall.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![TileId::WoodWall.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![None,                 TileId::WoodWall.s(),     TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::DungeonFloor.s(),                   TileId::DungeonFloor.s(), TileId::DungeonFloor.s(), TileId::WoodWall.s()],
                vec![None,                 None,                     TileId::WoodWall.s(),     TileId::WoodWall.s(),     TileId::DungeonFloor.i(ObjectId::Door),     TileId::WoodWall.s(),     TileId::WoodWall.s()],
            ],
            vec![((4,4), MobId::Undead)],
        ),
    }
}
