use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    algorithm::pathfinding::PathFindingQueue, prelude::*,
    render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use rand::Rng;

use crate::{tiles::TileData, utils::Map2D};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const TERRAIN_SIZE: u32 = 64;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;
const MOUNTAIN_RADIUS: f32 = 28.;
const DIRT_LAYER_SIZE: f32 = 6.;

#[derive(Clone, Copy)]
pub struct TilemapFile {
    name: &'static str,
    size: UVec2,
    start_atlas_index: i32,
}

const N: usize = 3;
pub type TF = (&'static str, UVec2);

pub struct TilemapFiles {
    files: [TilemapFile; N],
}

impl TilemapFiles {
    pub const T: Self = Self::new();

    pub const FLOORS: TF = ("floors", UVec2::new(2, 2));
    pub const WALLS: TF = ("walls", UVec2::new(2, 2));
    pub const FURNITURES: TF = ("furnitures", UVec2::new(2, 2));

    pub const fn new() -> Self {
        let tilemaps = [Self::FLOORS, Self::WALLS, Self::FURNITURES];

        let mut files = [TilemapFile {
            name: "",
            size: UVec2::ZERO,
            start_atlas_index: 0,
        }; N];

        let mut i = 0;
        let mut atlas_index = 0;

        while i < N {
            files[i] = TilemapFile {
                name: tilemaps[i].0,
                size: tilemaps[i].1,
                start_atlas_index: atlas_index,
            };
            atlas_index += (tilemaps[i].1.x * tilemaps[i].1.y) as i32;
            i += 1;
        }

        Self { files }
    }

    pub const fn atlas_index(&self, file: TF, atlas_index: i32) -> i32 {
        let mut i = 0;
        while i < N {
            // Can't compare strings directly, dirty workaround
            if self.files[i].name.len() == file.0.len() {
                return self.files[i].start_atlas_index + atlas_index;
            }
            i += 1;
        }
        0
    }
}

#[derive(Component)]
pub struct TilemapData(pub Map2D<TERRAIN_SIZE_USIZE, TileData>);

pub fn spawn_terrain(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();

    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("terrain".to_string()),
        tile_render_size: TileRenderSize(Vec2::splat(TILE_SIZE)),
        slot_size: TilemapSlotSize(Vec2::splat(TILE_SIZE)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::new(
            TilemapFiles::T
                .files
                .iter()
                .map(|file| {
                    TilemapTexture::new(
                        assets_server.load(format!("tilemaps/{}.png", file.name)),
                        TilemapTextureDescriptor::new(
                            file.size * TILE_SIZE_U,
                            UVec2::splat(TILE_SIZE_U),
                        ),
                    )
                })
                .collect(),
            FilterMode::Nearest,
        )),
        ..default()
    };

    let base_tile = TileData::GRASS_FLOOR;

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        base_tile.tile_builder(),
    );

    let mut tilemap_data = TilemapData(Map2D::new(base_tile));

    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            let dx = x as i32 - TERRAIN_SIZE as i32 / 2;
            let dy = y as i32 - TERRAIN_SIZE as i32 / 2;
            let distance = (dx * dx + dy * dy) as f32;

            if distance < MOUNTAIN_RADIUS.powi(2) {
                let mut rng = rand::thread_rng();
                let dirt_layer_size = rng.gen_range(DIRT_LAYER_SIZE * 0.5..DIRT_LAYER_SIZE);

                let tile = if distance < (MOUNTAIN_RADIUS - dirt_layer_size).powi(2) {
                    TileData::STONE_WALL
                } else {
                    TileData::DIRT_WALL
                };

                let index = IVec2::new(x as i32, y as i32);

                tile.set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );
            }
        }
    }

    let pathfinding_queue = PathFindingQueue::new_with_schedules(std::iter::empty());

    commands
        .entity(entity)
        .insert((tilemap, tilemap_data, pathfinding_queue));
}
