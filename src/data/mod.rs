use std::sync::LazyLock;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{BuildResult, MobData, ObjectData, TileData, enum_map};

mod macros;
mod structures;
pub use structures::*;

enum_map! {
    ObjectId => ObjectData {
        Wood = ObjectData::passable("wood"),
        Hide = ObjectData::passable("hide"),
        Tree = ObjectData::blocking_non_carriable("tree"),
        Table = ObjectData::blocking("table"),
        Stool = ObjectData::blocking("stool"),
        Bed = ObjectData::passable("bed"),
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
        Armor = ObjectData::armor("armor", 2),
        Scarecrow = ObjectData::blocking("scarecrow"),
        Haystack = ObjectData::blocking("haystack"),
        FishingSpot = ObjectData::passable_non_carriable("fishing_spot"),
        Fish = ObjectData::passable("fish"),
        WaterBucket = ObjectData::passable("water_bucket"),
        Bush = ObjectData::passable_non_carriable("bush"),
        BerryBush = ObjectData::passable_non_carriable("berry_bush"),
        Berries = ObjectData::passable("berries"),
        Honeycomb = ObjectData::passable("honeycomb"),
        Beehive = ObjectData::blocking("beehive"),
        MeadVat = ObjectData::blocking("mead_vat"),
        Hydromel = ObjectData::passable("hydromel"),
    }
}

enum_map! {
    TileId => TileData {
        GrassFloor = TileData::floor("grass"),
        StoneFloor = TileData::floor("stone"),
        DungeonFloor = TileData::floor("dungeon"),
        Bridge = TileData::floor("bridge"),
        SandFloor = TileData::floor("sand"),
        WoodFloor = TileData::floor("wood"),
        ShallowWater = TileData::floor("shallow_water"),

        Water = TileData::wall("water"),
        Lava = TileData::wall("lava"),
        DirtWall = TileData::wall("dirt"),
        StoneWall = TileData::wall("stone"),
        DungeonWall = TileData::wall("dungeon"),
        WoodWall = TileData::wall("wood"),
    }
}

enum_map! {
    MobId => MobData {
        Sheep = MobData::new("sheep", 2, 60.0, ObjectId::Hide),
        Boar = MobData::new("boar", 3, 50.0, ObjectId::Hide),
        Undead = MobData::new("undead", 5, 40.0, ObjectId::CopperIngot),
    }
}

#[rustfmt::skip]
pub const BUILD_RECIPES: &[(BuildResult, &[ObjectId])] = &[
    (BuildResult::Tile(TileId::WoodWall), &[ObjectId::Wood]),
    (BuildResult::Tile(TileId::WoodFloor), &[ObjectId::Wood]),
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
    (BuildResult::Object(ObjectId::Haystack), &[ObjectId::Wheat, ObjectId::Wheat, ObjectId::Wheat]),
    (BuildResult::Object(ObjectId::Bush), &[ObjectId::Berries]),
    (BuildResult::Object(ObjectId::Beehive), &[ObjectId::Wood, ObjectId::Honeycomb, ObjectId::Wood]),
    (BuildResult::Object(ObjectId::MeadVat), &[ObjectId::Wood, ObjectId::Wood, ObjectId::Wood, ObjectId::Honeycomb, ObjectId::Honeycomb]),
];

#[rustfmt::skip]
pub static WORKSTATIONS: LazyLock<HashMap<ObjectId, (ObjectId, Vec<ObjectId>)>> =
LazyLock::new(|| HashMap::from([
    (ObjectId::Furnace, (ObjectId::Bread, vec![ObjectId::Wheat, ObjectId::Wood])),
    (ObjectId::Forge, (ObjectId::CopperIngot, vec![ObjectId::CopperOre, ObjectId::CopperOre])),
    (ObjectId::Grindstone, (ObjectId::Sword, vec![ObjectId::CopperIngot, ObjectId::CopperIngot])),
    (ObjectId::Anvil, (ObjectId::Armor, vec![ObjectId::CopperIngot, ObjectId::CopperIngot, ObjectId::CopperIngot])),
    (ObjectId::MeadVat, (ObjectId::Hydromel, vec![ObjectId::Honeycomb, ObjectId::WaterBucket])),
]));

pub static EAT_VALUES: LazyLock<HashMap<ObjectId, i32>> = LazyLock::new(|| {
    HashMap::from([
        (ObjectId::Bread, 500),
        (ObjectId::Fish, 600),
        (ObjectId::Wheat, 50),
    ])
});

pub static SLEEP_VALUES: LazyLock<HashMap<ObjectId, i32>> = LazyLock::new(|| {
    HashMap::from([
        (ObjectId::Bed, 100),
        (ObjectId::Haystack, 60),
        (ObjectId::Stool, 10),
        (ObjectId::Table, 20),
    ])
});
