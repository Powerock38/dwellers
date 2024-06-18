use crate::{BuildResult, ObjectData, TileData};

impl TileData {
    pub const GRASS_FLOOR: Self = Self::floor(0);
    pub const STONE_FLOOR: Self = Self::floor(1);
    pub const DUNGEON_FLOOR: Self = Self::floor(2);
    pub const BRIDGE_FLOOR: Self = Self::floor(3);

    pub const DIRT_WALL: Self = Self::wall(0);
    pub const STONE_WALL: Self = Self::wall(1);
    pub const DUNGEON_WALL: Self = Self::wall(2);
    pub const WATER: Self = Self::wall(3);
}

impl ObjectData {
    pub const WOOD: Self = Self::passable(0);
    pub const RUG: Self = Self::passable(1);
    pub const TREE: Self = Self::blocking_non_carriable(2);
    pub const TABLE: Self = Self::blocking(3);
    pub const STOOL: Self = Self::blocking(4);
    pub const BED: Self = Self::blocking(5);
    pub const DOOR: Self = Self::passable(6);
    pub const ROCK: Self = Self::passable(7);
    pub const TALL_GRASS: Self = Self::passable_non_carriable(8);
    pub const SEEDS: Self = Self::passable(9);
    pub const FARM: Self = Self::passable_non_carriable(10);
    pub const WHEAT_PLANT: Self = Self::passable(11);
    pub const WHEAT: Self = Self::passable(12);
    pub const BREAD: Self = Self::passable(13); //TODO: draw this
    pub const FURNACE: Self = Self::blocking(14);
}

pub const BUILD_RECIPES: &[(&str, BuildResult, ObjectData)] = &[
    (
        "wall",
        BuildResult::Tile(TileData::DUNGEON_WALL),
        ObjectData::ROCK,
    ),
    (
        "table",
        BuildResult::Object(ObjectData::TABLE),
        ObjectData::WOOD,
    ),
    (
        "stool",
        BuildResult::Object(ObjectData::STOOL),
        ObjectData::WOOD,
    ),
    (
        "bed",
        BuildResult::Object(ObjectData::BED),
        ObjectData::WOOD,
    ),
    (
        "door",
        BuildResult::Object(ObjectData::DOOR),
        ObjectData::WOOD,
    ),
    (
        "farm",
        BuildResult::Object(ObjectData::FARM),
        ObjectData::SEEDS,
    ),
];
