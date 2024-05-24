use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use rand::Rng;

use crate::{tiles::TileData, utils::Map2D};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;

pub const TERRAIN_SIZE: u32 = 64;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;

const CHUNK_SIZE: u32 = 16;

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
pub struct TilemapData(pub Map2D<TileData>);

impl TilemapData {
    pub fn non_blocking_neighbours(&self, pos: IVec2) -> Vec<IVec2> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let index = pos + p;
                if let Some(tile) = self.0.get(index) {
                    if !tile.is_blocking() {
                        return Some(index);
                    }
                }

                None
            })
            .collect()
    }
}

pub fn spawn_terrain(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn((Camera2dBundle::default(), CameraChunkUpdater::new(1.3, 2.2)));

    let entity = commands.spawn_empty().id();

    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("terrain".to_string()),
        tile_render_size: TileRenderSize(Vec2::splat(TILE_SIZE)),
        slot_size: TilemapSlotSize(Vec2::splat(TILE_SIZE)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(CHUNK_SIZE, entity),
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

    let mut tilemap_data = TilemapData(Map2D::new(base_tile, TERRAIN_SIZE_USIZE));

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

    commands.entity(entity).insert((tilemap, tilemap_data));
}

/*
pub fn load_unload_chunks(
    mut commands: Commands,
    mut ev: EventReader<CameraChunkUpdation>,
    tilemap: Query<Entity, With<TilemapStorage>>,
    mut load_cache: ResMut<ChunkLoadCache>,
    mut save_cache: ResMut<ChunkSaveCache>,
) {
    let tilemap = tilemap.single();
    let mut to_load = Vec::new();
    let mut to_unload = Vec::new();

    ev.read().for_each(|e| match e {
        CameraChunkUpdation::Entered(_, chunk) => to_load.push(*chunk),
        CameraChunkUpdation::Left(_, chunk) => to_unload.push((*chunk, true)),
    });

    if !to_load.is_empty() {
        load_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::COLOR | TilemapLayer::PATH,
            to_load.into_iter(),
        );
    }

    if !to_unload.is_empty() {
        save_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::COLOR | TilemapLayer::PATH,
            to_unload.into_iter(),
        );
    }
}
 */
