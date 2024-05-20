use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    algorithm::pathfinding::{PathFindingQueue, PathTilemaps},
    prelude::*,
    render::material::StandardTilemapMaterial,
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        map::TilemapTextures,
    },
};
use rand::Rng;

use crate::{extract_ok, utils::Map2D};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const TERRAIN_SIZE: u32 = 64;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;
const MOUNTAIN_RADIUS: f32 = 28.;
const DIRT_LAYER_SIZE: f32 = 6.;

#[derive(Default, Clone, Copy)]
pub struct TileData {
    pub wall: bool,
}

#[derive(Component)]
pub struct TilemapData(pub Map2D<TERRAIN_SIZE_USIZE, TileData>);

#[derive(Event)]
pub struct MineTile(pub IVec2);

pub fn spawn_terrain(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();

    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("dungeon_terrain".to_string()),
        tile_render_size: TileRenderSize(Vec2::splat(TILE_SIZE)),
        slot_size: TilemapSlotSize(Vec2::splat(TILE_SIZE)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::new(
            vec![
                TilemapTexture::new(
                    assets_server.load("tilemaps/floors.png"),
                    TilemapTextureDescriptor::new(UVec2::new(32, 32), UVec2::splat(TILE_SIZE_U)),
                ),
                TilemapTexture::new(
                    assets_server.load("tilemaps/walls.png"),
                    TilemapTextureDescriptor::new(UVec2::new(32, 32), UVec2::splat(TILE_SIZE_U)),
                ),
            ],
            FilterMode::Nearest,
        )),
        ..default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        TileBuilder::new().with_layer(
            0, // grass floor
            TileLayer::no_flip(0),
        ),
    );

    let mut tilemap_data = TilemapData(Map2D::new());

    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            let dx = x as i32 - TERRAIN_SIZE as i32 / 2;
            let dy = y as i32 - TERRAIN_SIZE as i32 / 2;
            let distance = (dx * dx + dy * dy) as f32;

            if distance < MOUNTAIN_RADIUS.powi(2) {
                let mut rng = rand::thread_rng();
                let dirt = rng.gen_range(DIRT_LAYER_SIZE * 0.5..DIRT_LAYER_SIZE);

                let tile = if distance < (MOUNTAIN_RADIUS - dirt).powi(2) {
                    4 // stone wall
                } else {
                    5 // dirt wall
                };

                let index = IVec2::new(x as i32, y as i32);

                tilemap.storage.set(
                    &mut commands,
                    index,
                    TileBuilder::new().with_layer(0, TileLayer::no_flip(tile)),
                );

                tilemap_data.0.set(index, TileData { wall: true });
            }
        }
    }

    let pathfinding_queue = PathFindingQueue::new_with_schedules(std::iter::empty());

    commands
        .entity(entity)
        .insert((tilemap, tilemap_data, pathfinding_queue));
}

pub fn event_mine_tile(
    mut commands: Commands,
    mut ev_mine_tile: EventReader<MineTile>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    for MineTile(index) in ev_mine_tile.read() {
        if let Some(tile_data) = tilemap_data.0.get(*index) {
            if tile_data.wall {
                tilemap.set(
                    &mut commands,
                    *index,
                    TileBuilder::new().with_layer(0, TileLayer::no_flip(4)),
                );

                tilemap_data.0.set(*index, TileData { wall: false });
            }
        }
    }
}

pub fn update_path_tilemaps(
    q_tilemap: Query<(Entity, &TilemapData), Changed<TilemapData>>,
    mut path_tilemaps: ResMut<PathTilemaps>,
) {
    let (entity, tilemap_data) = extract_ok!(q_tilemap.get_single());

    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        |index| {
            if let Some(tile_data) = tilemap_data.0.get(index) {
                if !tile_data.wall {
                    return Some(PathTile { cost: 1 });
                }
            }
            None
        },
    );

    path_tilemaps.insert(entity, path_tilemap);
}
