use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::{
    map::{
        TilemapGridSize, TilemapId, TilemapRenderSettings, TilemapSize, TilemapTexture,
        TilemapTileSize,
    },
    tiles::{TileBundle, TileColor, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};

use crate::{tilemap_data::TilemapData, ObjectData, TileData, TilePlaced};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const CHUNK_SIZE: u32 = 64;
const RENDER_CHUNK_SIZE: u32 = CHUNK_SIZE * 2;

#[derive(Component)]
pub struct ChunkTileLayer;

#[derive(Component)]
pub struct ChunkObjectLayer;

pub fn init_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TilemapData::default());

    let mut textures = Vec::new();
    for dirname in &["objects", "walls", "floors"] {
        for entry in std::fs::read_dir(format!("assets/tiles/{dirname}")).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "png" {
                let path = path.strip_prefix("assets/").unwrap().to_owned();
                textures.push(asset_server.load(path));
            }
        }
    }

    commands.insert_resource(TilemapTextures::new(TilemapTexture::Vector(textures)));
}

pub fn manage_chunks(
    mut commands: Commands,
    q_chunks_tile_layer: Query<
        (Entity, &Transform),
        (With<ChunkTileLayer>, Without<ChunkObjectLayer>),
    >,
    q_chunks_object_layer: Query<
        (Entity, &Transform),
        (With<ChunkObjectLayer>, Without<ChunkTileLayer>),
    >,
    mut tilemap_data: ResMut<TilemapData>,
    tilemap_textures: Res<TilemapTextures>,
) {
    let mut created_chunks = Vec::new();

    for (index, _) in &tilemap_data.tiles_to_update {
        let chunk_index = TilemapData::index_to_chunk(*index).0;

        if created_chunks.contains(&chunk_index) {
            continue;
        }

        // Check if the bevy_ecs_tilemap chunk exists
        if q_chunks_tile_layer
            .iter()
            .any(|(_, t)| chunk_index_is_translation(chunk_index, t.translation))
        {
            continue;
        }

        // If the chunk doesn't exist, create it

        let pos = Vec2::new(
            chunk_index.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
            chunk_index.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
        ) + TILE_SIZE / 2.0;

        // Tile layer
        commands.spawn((
            ChunkTileLayer,
            new_tilemap(tilemap_textures.textures.clone(), pos.extend(0.0)),
        ));

        // Object layer
        commands.spawn((
            ChunkObjectLayer,
            new_tilemap(tilemap_textures.textures.clone(), pos.extend(1.0)),
        ));

        created_chunks.push(chunk_index);
    }

    let chunks_to_remove = tilemap_data.chunks_to_remove.drain(..).collect::<Vec<_>>();

    for chunk_index in chunks_to_remove {
        if let Some((entity, _)) = q_chunks_tile_layer
            .iter()
            .find(|(_, t)| chunk_index_is_translation(chunk_index, t.translation))
        {
            commands.entity(entity).despawn_recursive();
        }

        if let Some((entity, _)) = q_chunks_object_layer
            .iter()
            .find(|(_, t)| chunk_index_is_translation(chunk_index, t.translation))
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn new_tilemap(texture: TilemapTexture, pos: Vec3) -> TilemapBundle {
    TilemapBundle {
        grid_size: TilemapGridSize::new(TILE_SIZE, TILE_SIZE),
        size: TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE),
        storage: TileStorage::empty(TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE)),
        texture,
        tile_size: TilemapTileSize::new(TILE_SIZE, TILE_SIZE),
        transform: Transform::from_translation(pos),
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::splat(RENDER_CHUNK_SIZE),
            ..default()
        },
        ..default()
    }
}

fn chunk_index_is_translation(chunk_index: IVec2, translation: Vec3) -> bool {
    ((translation.truncate() - TILE_SIZE / 2.) / (CHUNK_SIZE as f32 * TILE_SIZE)).as_ivec2()
        == chunk_index
}

pub fn update_tilemap_from_data(
    mut commands: Commands,
    mut q_chunks_tile_layer: Query<
        (Entity, &mut TileStorage, &Transform),
        (With<ChunkTileLayer>, Without<ChunkObjectLayer>),
    >,
    mut q_chunks_object_layer: Query<
        (Entity, &mut TileStorage, &Transform),
        (With<ChunkObjectLayer>, Without<ChunkTileLayer>),
    >,
    mut tilemap_data: ResMut<TilemapData>,
    mut tilemap_textures: ResMut<TilemapTextures>,
) {
    let tiles_to_update = tilemap_data.tiles_to_update.drain().collect::<Vec<_>>();

    for (index, tile) in tiles_to_update {
        let chunk_index = TilemapData::index_to_chunk(index).0;
        let tile_pos = TilePos {
            x: index.x as u32 % CHUNK_SIZE,
            y: index.y as u32 % CHUNK_SIZE,
        };

        // retrieve Tile layer
        let Some((tile_layer_entity, mut tile_layer_chunk_storage, _)) = q_chunks_tile_layer
            .iter_mut()
            .find(|(_, _, t)| chunk_index_is_translation(chunk_index, t.translation))
        else {
            error!("Chunk not found for tile at index {:?}", index);
            continue;
        };

        // retrieve Object layer
        let Some((object_layer_entity, mut object_layer_chunk_storage, _)) = q_chunks_object_layer
            .iter_mut()
            .find(|(_, _, t)| chunk_index_is_translation(chunk_index, t.translation))
        else {
            error!("Chunk not found for object at index {:?}", index);
            continue;
        };

        // Lighting: darken tiles surrounded by walls
        let color = if tilemap_data
            .neighbours(index)
            .iter()
            .any(|(_, TilePlaced { id: tile, .. })| tile.is_transparent())
        {
            TileColor::default()
        } else {
            TileColor(Color::BLACK.with_alpha(0.5))
        };

        // add or update tile
        if let Some(tile_entity) = tile_layer_chunk_storage.get(&tile_pos) {
            if let Some(mut ec) = commands.get_entity(tile_entity) {
                ec.insert((tilemap_textures.get_atlas_index_tile(tile.id.data()), color));
            }
        } else {
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tile_layer_entity),
                    texture_index: tilemap_textures.get_atlas_index_tile(tile.id.data()),
                    color,
                    ..default()
                })
                .id();

            commands.entity(tile_layer_entity).add_child(tile_entity);
            tile_layer_chunk_storage.set(&tile_pos, tile_entity);
        };

        // add, update or remove object
        if let Some(object) = tile.object {
            if let Some(object_entity) = object_layer_chunk_storage.get(&tile_pos) {
                commands.entity(object_entity).try_insert((
                    tilemap_textures.get_atlas_index_object(object.data()),
                    color,
                ));
            } else {
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(object_layer_entity),
                        texture_index: tilemap_textures.get_atlas_index_object(object.data()),
                        color,
                        ..default()
                    })
                    .id();

                commands.entity(object_layer_entity).add_child(tile_entity);
                object_layer_chunk_storage.set(&tile_pos, tile_entity);
            };
        } else if let Some(object_entity) = object_layer_chunk_storage.get(&tile_pos) {
            commands.entity(object_entity).despawn_recursive();
            object_layer_chunk_storage.remove(&tile_pos);
        }
    }
}

#[derive(Resource)]
pub struct TilemapTextures {
    pub textures: TilemapTexture,
    cache: HashMap<(&'static str, &'static str), TileTextureIndex>,
}

impl TilemapTextures {
    pub fn new(textures: TilemapTexture) -> Self {
        Self {
            textures,
            cache: HashMap::default(),
        }
    }

    pub fn get_atlas_index_tile(&mut self, tile: TileData) -> TileTextureIndex {
        let folder = if tile.is_wall() { "walls" } else { "floors" };
        self.get_atlas_index(folder, tile.filename())
    }

    pub fn get_atlas_index_object(&mut self, object: ObjectData) -> TileTextureIndex {
        self.get_atlas_index("objects", object.filename())
    }

    fn get_atlas_index(
        &mut self,
        folder: &'static str,
        filename: &'static str,
    ) -> TileTextureIndex {
        if let Some(index) = self.cache.get(&(folder, filename)) {
            return *index;
        }

        debug!("Loading texture: {folder}/{filename}");

        let index = TileTextureIndex(match &self.textures {
            TilemapTexture::Vector(textures) => textures
                .iter()
                .position(|t| {
                    let paths_string = t.path().unwrap().to_string();
                    paths_string.ends_with(format!("{folder}/{filename}.png").as_str())
                        || paths_string.ends_with(format!("{folder}\\{filename}.png").as_str())
                })
                .unwrap() as u32,

            _ => 0,
        });

        self.cache.insert((folder, filename), index);
        index
    }
}
