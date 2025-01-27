use std::sync::LazyLock;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bitcode::{Decode, Encode};

use crate::{enum_map, structures::StructureData, BuildResult, MobData, ObjectData, TileData};

enum_map! {
    ObjectId => ObjectData {
        Wood = ObjectData::passable("wood"),
        Hide = ObjectData::passable("hide"),
        Tree = ObjectData::blocking_non_carriable("tree"),
        Table = ObjectData::blocking("table"),
        Stool = ObjectData::blocking("stool"),
        Bed = ObjectData::blocking("bed"),
        Door = ObjectData::passable("door"),
        Rock = ObjectData::passable("rock"),
        TallGrass = ObjectData::passable_non_carriable("tall_grass"),
        Seeds = ObjectData::passable("seeds"),
        Farm = ObjectData::passable_non_carriable("farm"),
        WheatPlant = ObjectData::passable_non_carriable("wheat_plant"),
        Wheat = ObjectData::passable("wheat"),
        Furnace = ObjectData::blocking("furnace"),
        Bread = ObjectData::passable("bread"),
        PalmTree = ObjectData::blocking_non_carriable("palm_tree"),
        Cactus = ObjectData::passable_non_carriable("cactus"),
        CopperOre = ObjectData::passable("copper_ore"),
        CopperIngot = ObjectData::passable("copper_ingot"),
        Forge = ObjectData::blocking("forge"),
        Anvil = ObjectData::blocking("anvil"),
        Grindstone = ObjectData::blocking("grindstone"),
        Sword = ObjectData::tool("sword"),
        Armor = ObjectData::armor("armor"),
        Scarecrow = ObjectData::blocking("scarecrow"),
    }
}

enum_map! {
    TileId => TileData {
        GrassFloor = TileData::floor("grass"),
        StoneFloor = TileData::floor("stone"),
        DungeonFloor = TileData::floor("dungeon"),
        Bridge = TileData::floor("bridge"),
        SandFloor = TileData::floor("sand"),

        DirtWall = TileData::wall("dirt"),
        StoneWall = TileData::wall("stone"),
        DungeonWall = TileData::wall("dungeon"),
        Water = TileData::wall("water"),
        WoodWall = TileData::wall("wood"),
    }
}

enum_map! {
    MobId => MobData {
        Sheep = MobData::new("sheep", 60.0, ObjectId::Hide),
        Boar = MobData::new("boar", 50.0, ObjectId::Hide),
        Undead = MobData::new("undead", 40.0, ObjectId::CopperIngot),
    }
}

#[rustfmt::skip]
pub const BUILD_RECIPES: &[(BuildResult, &[ObjectId])] = &[
    (BuildResult::Tile(TileId::WoodWall), &[ObjectId::Wood]),
    (BuildResult::Tile(TileId::DungeonWall), &[ObjectId::Rock]),
    (BuildResult::Tile(TileId::Bridge), &[ObjectId::Wood]),
    (BuildResult::Object(ObjectId::Table), &[ObjectId::Wood, ObjectId::Wood]),
    (BuildResult::Object(ObjectId::Stool), &[ObjectId::Wood]),
    (BuildResult::Object(ObjectId::Bed), &[ObjectId::Wood]),
    (BuildResult::Object(ObjectId::Door), &[ObjectId::Wood]),
    (BuildResult::Object(ObjectId::Farm), &[ObjectId::Seeds]),
    (BuildResult::Object(ObjectId::Scarecrow), &[ObjectId::Wood, ObjectId::Wheat, ObjectId::Wheat]),
    (BuildResult::Object(ObjectId::Furnace), &[ObjectId::Rock, ObjectId::Rock, ObjectId::Rock]),
    (BuildResult::Object(ObjectId::Forge), &[ObjectId::Rock, ObjectId::Rock, ObjectId::Rock, ObjectId::CopperOre, ObjectId::CopperOre]),
    (BuildResult::Object(ObjectId::Anvil), &[ObjectId::CopperIngot, ObjectId::CopperIngot, ObjectId::CopperIngot, ObjectId::CopperIngot]),
    (BuildResult::Object(ObjectId::Grindstone), &[ObjectId::Rock, ObjectId::Wood]),
];

#[rustfmt::skip]
pub static WORKSTATIONS: LazyLock<HashMap<ObjectId, (ObjectId, Vec<ObjectId>)>> =
LazyLock::new(|| HashMap::from([
    (ObjectId::Furnace, (ObjectId::Bread, vec![ObjectId::Wheat, ObjectId::Wood])),
    (ObjectId::Forge, (ObjectId::CopperIngot, vec![ObjectId::CopperOre, ObjectId::CopperOre])),
    (ObjectId::Grindstone, (ObjectId::Sword, vec![ObjectId::CopperIngot, ObjectId::CopperIngot])),
    (ObjectId::Anvil, (ObjectId::Armor, vec![ObjectId::CopperIngot, ObjectId::CopperIngot, ObjectId::CopperIngot])),
]));

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
