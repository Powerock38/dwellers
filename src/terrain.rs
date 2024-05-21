use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    algorithm::pathfinding::PathFindingQueue, prelude::*,
    render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use rand::Rng;

use crate::{
    tiles::{set_tile, TileData},
    utils::Map2D,
};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const TERRAIN_SIZE: u32 = 64;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;
const MOUNTAIN_RADIUS: f32 = 28.;
const DIRT_LAYER_SIZE: f32 = 6.;

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
            vec![
                TilemapTexture::new(
                    assets_server.load("tilemaps/floors.png"),
                    TilemapTextureDescriptor::new(
                        UVec2::new(2, 2) * TILE_SIZE_U,
                        UVec2::splat(TILE_SIZE_U),
                    ),
                ),
                TilemapTexture::new(
                    assets_server.load("tilemaps/walls.png"),
                    TilemapTextureDescriptor::new(
                        UVec2::new(2, 2) * TILE_SIZE_U,
                        UVec2::splat(TILE_SIZE_U),
                    ),
                ),
            ],
            FilterMode::Nearest,
        )),
        ..default()
    };

    let base_tile = TileData::GRASS_FLOOR;

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        TileBuilder::new().with_layer(0, base_tile.layer()),
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

                set_tile(
                    index,
                    tile,
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
