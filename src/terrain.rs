use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use noise::{NoiseFn, Perlin, RidgedMulti};

use crate::{
    tiles::{ObjectData, TileData},
    utils::Map2D,
};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;

pub const TERRAIN_SIZE: u32 = 256;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;

const CHUNK_SIZE: u32 = 16;

// World Generation
const TREE_NOISE_SCALE: f64 = 10.0;
const MOUNTAIN_NOISE_SCALE: f64 = 5.0;

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
    pub const WALLS: TF = ("walls", UVec2::new(4, 4));
    pub const OBJECTS: TF = ("objects", UVec2::new(4, 4));

    pub const fn new() -> Self {
        let tilemaps = [Self::FLOORS, Self::WALLS, Self::OBJECTS];

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
    pub fn neighbours(&self, pos: IVec2) -> Vec<(IVec2, TileData)> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let index = pos + p;

                if let Some(tile) = self.0.get(index) {
                    return Some((index, tile));
                }

                None
            })
            .collect()
    }

    pub fn non_blocking_neighbours_pos(&self, pos: IVec2) -> Vec<IVec2> {
        self.neighbours(pos)
            .into_iter()
            .filter_map(|(index, tile)| {
                if !tile.is_blocking() {
                    return Some(index);
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
    commands.spawn((
        Camera2dBundle::default(), /*CameraChunkUpdater::new(1.3, 2.2) */
    ));

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

    let seed = rand::random();

    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            let index = IVec2::new(x as i32, y as i32);
            let u = x as f64 / TERRAIN_SIZE as f64;
            let v = y as f64 / TERRAIN_SIZE as f64;

            let noise = RidgedMulti::<Perlin>::new(seed);

            // Mountains
            let mountain_noise_value =
                noise.get([u * MOUNTAIN_NOISE_SCALE, v * MOUNTAIN_NOISE_SCALE]);
            if mountain_noise_value < -0.1 {
                let tile = if mountain_noise_value < -0.3 {
                    TileData::STONE_WALL
                } else {
                    TileData::DIRT_WALL
                };

                tile.set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );

                continue;
            }

            // Rivers
            if mountain_noise_value > 0.5 {
                TileData::WATER.set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );

                continue;
            }

            // Trees
            let tree_noise_value = noise.get([u * TREE_NOISE_SCALE, v * TREE_NOISE_SCALE]);
            if tree_noise_value > 0.0 {
                // 0.5 {
                TileData::GRASS_FLOOR.with(ObjectData::TREE).set_at(
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

pub fn find_from_center(index: IVec2, is_valid: impl Fn(IVec2) -> bool) -> Option<IVec2> {
    for radius in 0..=(TERRAIN_SIZE as i32 / 2) {
        // Top and Bottom edges of the square
        for x in (index.x - radius).max(1)..=(index.x + radius).min(TERRAIN_SIZE as i32 - 2) {
            // Top edge
            let top_y = (index.y - radius).max(1);
            if is_valid(IVec2::new(x, top_y)) {
                return Some(IVec2::new(x, top_y));
            }
            // Bottom edge
            let bottom_y = (index.y + radius).min(TERRAIN_SIZE as i32 - 2);
            if is_valid(IVec2::new(x, bottom_y)) {
                return Some(IVec2::new(x, bottom_y));
            }
        }

        // Left and Right edges of the square (excluding corners already checked)
        for y in
            ((index.y - radius + 1).max(1))..=((index.y + radius - 1).min(TERRAIN_SIZE as i32 - 2))
        {
            // Left edge
            let left_x = (index.x - radius).max(1);
            if is_valid(IVec2::new(left_x, y)) {
                return Some(IVec2::new(left_x, y));
            }
            // Right edge
            let right_x = (index.x + radius).min(TERRAIN_SIZE as i32 - 2);
            if is_valid(IVec2::new(right_x, y)) {
                return Some(IVec2::new(right_x, y));
            }
        }
    }

    None
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
