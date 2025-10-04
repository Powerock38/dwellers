use std::sync::LazyLock;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{BuildResult, MobData, Object, Tile, enum_map};

mod macros;
mod structures;
pub use structures::*;

enum_map! {
    ObjectId => Object {
        Wood = Object::passable("wood"),
        Hide = Object::passable("hide"),
        Tree = Object::blocking_non_carriable("tree"),
        Table = Object::blocking("table"),
        Stool = Object::blocking("stool"),
        Bed = Object::passable("bed"),
        Door = Object::passable("door"),
        Rock = Object::passable("rock"),
        TallGrass = Object::passable_non_carriable("tall_grass"),
        Seeds = Object::passable("seeds"),
        Farm = Object::passable_non_carriable("farm"),
        WheatPlant = Object::passable_non_carriable("wheat_plant"),
        Wheat = Object::passable("wheat"),
        Furnace = Object::blocking("furnace"),
        Bread = Object::passable("bread"),
        PalmTree = Object::blocking_non_carriable("palm_tree"),
        Cactus = Object::passable_non_carriable("cactus"),
        CopperOre = Object::passable("copper_ore"),
        CopperIngot = Object::passable("copper_ingot"),
        Forge = Object::blocking("forge"),
        Anvil = Object::blocking("anvil"),
        Grindstone = Object::blocking("grindstone"),
        Sword = Object::tool("sword"),
        Armor = Object::armor("armor", 2),
        Scarecrow = Object::blocking("scarecrow"),
        Haystack = Object::blocking("haystack"),
        FishingSpot = Object::passable_non_carriable("fishing_spot"),
        Fish = Object::passable("fish"),
        WaterBucket = Object::passable("water_bucket"),
        Bush = Object::passable_non_carriable("bush"),
        BerryBush = Object::passable_non_carriable("berry_bush"),
        Berries = Object::passable("berries"),
        Honeycomb = Object::passable("honeycomb"),
        Beehive = Object::blocking("beehive"),
        MeadVat = Object::blocking("mead_vat"),
        Hydromel = Object::passable("hydromel"),
        MobLair = Object::blocking_non_carriable("mob_lair"),
    }
}

enum_map! {
    TileId => Tile {
        GrassFloor = Tile::floor("grass"),
        StoneFloor = Tile::floor("stone"),
        DungeonFloor = Tile::floor("dungeon"),
        Bridge = Tile::floor("bridge"),
        SandFloor = Tile::floor("sand"),
        WoodFloor = Tile::floor("wood"),
        ShallowWater = Tile::floor("shallow_water"),

        Water = Tile::wall("water"),
        Lava = Tile::wall("lava"),
        DirtWall = Tile::wall("dirt"),
        StoneWall = Tile::wall("stone"),
        DungeonWall = Tile::wall("dungeon"),
        WoodWall = Tile::wall("wood"),
    }
}

enum_map! {
    MobId => MobData {
        Sheep = MobData::new("sheep", 2, 60.0, ObjectId::Hide),
        Boar = MobData::new("boar", 3, 50.0, ObjectId::Hide),
        Undead = MobData::new("undead", 5, 40.0, ObjectId::CopperIngot),
        Snake = MobData::new("snake", 1, 70.0, ObjectId::Hide),
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
