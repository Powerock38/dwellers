use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};

use crate::utils::BoolMap;

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const TERRAIN_SIZE: u32 = 64;
const TERRAIN_SIZE_USIZE: usize = TERRAIN_SIZE as usize;
const MOUNTAIN_RADIUS: f32 = 28.;
const DIRT_LAYER_SIZE: f32 = 4.;

#[derive(Resource)]
pub struct Walkable(pub BoolMap<TERRAIN_SIZE_USIZE>);

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
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    let mut walkable = Walkable(BoolMap::new());

    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            let dx = x as i32 - TERRAIN_SIZE as i32 / 2;
            let dy = y as i32 - TERRAIN_SIZE as i32 / 2;
            let distance = (dx * dx + dy * dy) as f32;

            if distance < MOUNTAIN_RADIUS.powi(2) {
                let tile = if distance < (MOUNTAIN_RADIUS - DIRT_LAYER_SIZE).powi(2) {
                    4
                } else {
                    5
                };

                tilemap.storage.set(
                    &mut commands,
                    IVec2::new(x as i32, y as i32),
                    TileBuilder::new().with_layer(0, TileLayer::no_flip(tile)),
                );
            } else {
                walkable.0.set(x as usize, y as usize, true);
            }
        }
    }

    commands.insert_resource(walkable);

    commands.entity(entity).insert(tilemap);
}
