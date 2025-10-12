use bevy::{
    prelude::*,
    sprite_render::{AlphaMode2d, TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{
    ChunkWeatherMaterial, SaveName, SaveScoped, TilePlaced, TilemapData, Tileset,
    utils::transform_to_pos,
};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const CHUNK_SIZE: u32 = 64;

#[derive(Component, Default)]
#[require(SaveScoped)]
pub struct ChunkLayer;

#[derive(Component)]
#[require(ChunkLayer)]
pub struct ChunkTileLayer;

#[derive(Component)]
#[require(ChunkLayer)]
pub struct ChunkObjectLayer;

#[derive(Component)]
#[require(ChunkLayer)]
pub struct ChunkWeatherLayer;

fn new_tilemap(tileset: Handle<Image>, pos: Vec3) -> impl Bundle {
    (
        TilemapChunk {
            chunk_size: UVec2::splat(CHUNK_SIZE),
            tile_display_size: UVec2::splat(TILE_SIZE_U),
            tileset,
            alpha_mode: AlphaMode2d::Blend,
        },
        TilemapChunkTileData(vec![None; (CHUNK_SIZE * CHUNK_SIZE) as usize]),
        Transform::from_translation(pos),
    )
}

fn chunk_pos_is_transform(chunk_pos: IVec2, transform: &Transform) -> bool {
    let pos = transform_to_pos(transform);
    pos.div_euclid(IVec2::splat(CHUNK_SIZE as i32)) == chunk_pos
}

pub fn manage_chunks(
    mut commands: Commands,
    tilemap_textures: If<Res<Tileset>>,
    mut tilemap_data: ResMut<TilemapData>,
    save_name: Res<SaveName>,
    q_chunk_layers: Query<(Entity, &Transform), With<ChunkLayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkWeatherMaterial>>,
) {
    let mut created_chunks = Vec::new();

    for (pos, _) in &tilemap_data.tiles_to_update {
        let chunk_pos = TilemapData::pos_to_chunk_pos_and_local_index(*pos).0;

        if created_chunks.contains(&chunk_pos) {
            continue;
        }

        // Check if the TilemapChunk exists
        if q_chunk_layers
            .iter()
            .any(|(_, t)| chunk_pos_is_transform(chunk_pos, t))
        {
            continue;
        }

        // If the chunk doesn't exist, create it

        let chunk_pos_f32 = Vec2::new(
            chunk_pos.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
            chunk_pos.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
        ) + Vec2::splat((CHUNK_SIZE as f32 * TILE_SIZE) / 2.0);

        // Tile layer
        commands.spawn((
            ChunkTileLayer,
            new_tilemap(tilemap_textures.texture.clone(), chunk_pos_f32.extend(0.0)),
        ));

        // Object layer
        commands.spawn((
            ChunkObjectLayer,
            new_tilemap(tilemap_textures.texture.clone(), chunk_pos_f32.extend(1.0)),
        ));

        // Weather layer
        commands.spawn((
            ChunkWeatherLayer,
            Mesh2d(meshes.add(Rectangle::from_length(CHUNK_SIZE as f32 * TILE_SIZE))),
            MeshMaterial2d(materials.add(ChunkWeatherMaterial::new(save_name.seed()))),
            Transform::from_translation(chunk_pos_f32.extend(100.0)),
        ));

        created_chunks.push(chunk_pos);
    }

    let chunks_to_remove = tilemap_data.chunks_to_remove.drain(..).collect::<Vec<_>>();

    for chunk_pos in chunks_to_remove {
        for (entity, transform) in q_chunk_layers {
            if chunk_pos_is_transform(chunk_pos, transform) {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn update_tilemap_from_data(
    mut q_chunks_tile_layer: Query<
        (&mut TilemapChunkTileData, &Transform),
        (With<ChunkTileLayer>, Without<ChunkObjectLayer>),
    >,
    mut q_chunks_object_layer: Query<
        (&mut TilemapChunkTileData, &Transform),
        (With<ChunkObjectLayer>, Without<ChunkTileLayer>),
    >,
    mut tilemap_data: ResMut<TilemapData>,
    mut tilemap_textures: If<ResMut<Tileset>>,
) {
    let tiles_to_update = tilemap_data.tiles_to_update.drain().collect::<Vec<_>>();

    for (pos, tile) in tiles_to_update {
        let (chunk_pos, tile_index) = TilemapData::pos_to_chunk_pos_and_local_index(pos);

        // retrieve Tile layer
        let Some((mut tile_layer_chunk_data, _)) = q_chunks_tile_layer
            .iter_mut()
            .find(|(_, t)| chunk_pos_is_transform(chunk_pos, t))
        else {
            error!("Chunk not found for tile at pos {:?}", pos);
            continue;
        };

        // retrieve Object layer
        let Some((mut object_layer_chunk_data, _)) = q_chunks_object_layer
            .iter_mut()
            .find(|(_, t)| chunk_pos_is_transform(chunk_pos, t))
        else {
            error!("Chunk not found for object at pos {:?}", pos);
            continue;
        };

        // Lighting: darken tiles surrounded by walls
        let color = if tilemap_data
            .neighbours(pos)
            .iter()
            .any(|(_, TilePlaced { id: tile, .. })| tile.is_transparent())
        {
            Color::WHITE
        } else {
            Color::BLACK.with_alpha(0.8)
        };

        tile_layer_chunk_data[tile_index] = Some(TileData {
            tileset_index: tilemap_textures.get_atlas_index_tile(tile.id.data()),
            color,
            ..default()
        });

        // add, update or remove object
        if let Some(object) = tile.object {
            object_layer_chunk_data[tile_index] = Some(TileData {
                tileset_index: tilemap_textures.get_atlas_index_object(object.data()),
                color,
                ..default()
            });
        } else if object_layer_chunk_data[tile_index].is_some() {
            object_layer_chunk_data[tile_index] = None;
        }
    }
}
