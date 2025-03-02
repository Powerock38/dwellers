use std::sync::LazyLock;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    data::{MobId, ObjectId, TileId},
    enum_map, structure_ascii,
    structures::StructureData,
    tiles::TilePlaced,
};

pub static ASCII_TILES: LazyLock<HashMap<char, TilePlaced>> = LazyLock::new(|| {
    HashMap::from([
        (',', TileId::GrassFloor.place()),
        ('#', TileId::DungeonWall.place()),
        ('.', TileId::DungeonFloor.place()),
        ('D', TileId::DungeonFloor.with(ObjectId::Door)),
        ('Ď', TileId::WoodFloor.with(ObjectId::Door)),
        ('H', TileId::GrassFloor.with(ObjectId::Haystack)),
        ('=', TileId::WoodWall.place()),
        ('-', TileId::WoodFloor.place()),
        ('~', TileId::Water.place()),
    ])
});

enum_map! {
    StructureId => StructureData {

        DungeonCircleRoom = structure_ascii!(
            "
  #####
 #.....#
#.......#
#.......#
#.......#
#.......#
 #.....#
  ##D##
            ",
            vec![(4, 4, MobId::Undead)]
        ),

        Outpost = structure_ascii!(
            "
,,,,,,,,,
,====,,,,,,,,,,,,
,=--=,,,,,HH,,,,,
,=--=,,,HHH,,,,,,
,=Ď==,,,,,,,,,,,,
,,,,,,,,,,,,,
            ",
            vec![]
        ),
    }
}
