use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_ecs_tilemap::{
    map::{
        TilemapGridSize, TilemapId, TilemapRenderSettings, TilemapSize, TilemapTexture,
        TilemapTileSize,
    },
    prelude::MaterialTilemap,
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    MaterialTilemapBundle, TilemapBundle,
};

use crate::{tilemap_data::TilemapData, ObjectData, TileData};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const CHUNK_SIZE: u32 = 64;
const RENDER_CHUNK_SIZE: u32 = CHUNK_SIZE * 2;

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct TilemapMaterial {
    #[uniform(0)]
    brightness: f32,
}

impl MaterialTilemap for TilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "tilemap.wgsl".into()
    }
}

#[derive(Component)]
pub struct ChunkTileLayer;

#[derive(Component)]
pub struct ChunkObjectLayer;

pub fn init_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
) {
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

    let material = materials.add(TilemapMaterial { brightness: 0.5 });

    commands.insert_resource(TilemapTextures::new(
        TilemapTexture::Vector(textures),
        material,
    ));
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

    for (index, _) in &tilemap_data.tiles_to_spawn {
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
        let tile_layer_entity = commands.spawn_empty().id();

        let mut tile_storage = TileStorage::empty(TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE));

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tile_layer_entity),
                        ..default()
                    })
                    .id();
                commands.entity(tile_layer_entity).add_child(tile_entity);
                tile_storage.set(&tile_pos, tile_entity);
            }
        }

        commands.entity(tile_layer_entity).insert((
            ChunkTileLayer,
            MaterialTilemapBundle {
                material: tilemap_textures.material.clone(),
                grid_size: TilemapGridSize::new(TILE_SIZE, TILE_SIZE),
                size: TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE),
                storage: tile_storage,
                texture: tilemap_textures.textures.clone(),
                tile_size: TilemapTileSize::new(TILE_SIZE, TILE_SIZE),
                transform: Transform::from_translation(pos.extend(0.0)),
                render_settings: TilemapRenderSettings {
                    render_chunk_size: UVec2::splat(RENDER_CHUNK_SIZE),
                    ..default()
                },
                ..default()
            },
        ));

        // Object layer
        let tile_object_entity = commands.spawn_empty().id();

        let tile_storage = TileStorage::empty(TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE));

        commands.entity(tile_object_entity).insert((
            ChunkObjectLayer,
            new_tilemap(
                tilemap_textures.textures.clone(),
                tile_storage,
                pos.extend(1.0),
            ),
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

fn new_tilemap(texture: TilemapTexture, tile_storage: TileStorage, pos: Vec3) -> TilemapBundle {
    TilemapBundle {
        grid_size: TilemapGridSize::new(TILE_SIZE, TILE_SIZE),
        size: TilemapSize::new(CHUNK_SIZE, CHUNK_SIZE),
        storage: tile_storage,
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
    q_chunks_tile_layer: Query<
        (&TileStorage, &Transform),
        (With<ChunkTileLayer>, Without<ChunkObjectLayer>),
    >,
    mut q_chunks_object_layer: Query<
        (Entity, &mut TileStorage, &Transform),
        (With<ChunkObjectLayer>, Without<ChunkTileLayer>),
    >,
    mut tilemap_data: ResMut<TilemapData>,
    tilemap_textures: Res<TilemapTextures>,
) {
    let tiles_to_spawn = tilemap_data.tiles_to_spawn.drain(..).collect::<Vec<_>>();

    for (index, tile) in tiles_to_spawn {
        let chunk_index = TilemapData::index_to_chunk(index).0;

        // Tile layer
        let Some((tile_layer_chunk_storage, _)) = q_chunks_tile_layer
            .iter()
            .find(|(_, t)| chunk_index_is_translation(chunk_index, t.translation))
        else {
            error!("Chunk not found for tile at index {:?}", index);
            continue;
        };

        let Some(tile_entity) = tile_layer_chunk_storage.get(&TilePos {
            x: index.x as u32 % CHUNK_SIZE,
            y: index.y as u32 % CHUNK_SIZE,
        }) else {
            error!("Tile entity not found for tile at index {:?}", index);
            continue;
        };

        commands
            .entity(tile_entity)
            .insert(tilemap_textures.get_atlas_index_tile(tile.id.data()));

        // Object layer
        let tile_pos = TilePos {
            x: index.x as u32 % CHUNK_SIZE,
            y: index.y as u32 % CHUNK_SIZE,
        };

        let Some((object_layer_entity, mut object_layer_chunk_storage, _)) = q_chunks_object_layer
            .iter_mut()
            .find(|(_, _, t)| chunk_index_is_translation(chunk_index, t.translation))
        else {
            error!("Chunk not found for object at index {:?}", index);
            continue;
        };

        if let Some(object) = tile.object {
            if let Some(object_entity) = object_layer_chunk_storage.get(&tile_pos) {
                commands
                    .entity(object_entity)
                    .insert(tilemap_textures.get_atlas_index_object(object.data()));
            } else {
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(object_layer_entity),
                        texture_index: tilemap_textures.get_atlas_index_object(object.data()),
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

        // TODO Update light level
        // for index in tilemap_data
        //     .neighbours(index)
        //     .iter()
        //     .map(|n| n.0)
        //     .chain([index])
        // {
        //     let tint =
        //         if tilemap_data
        //             .neighbours(index)
        //             .iter()
        //             .all(|(_, TilePlaced { id: tile, .. })| {
        //                 tile.data().is_wall() && *tile != TileId::Water
        //             })
        //         {
        //             Color::BLACK
        //         } else {
        //             Color::WHITE
        //         };

        //     tilemap.update(
        //         &mut commands,
        //         index,
        //         TileUpdater {
        //             tint: Some(tint.into()),
        //             ..default()
        //         },
        //     );
        // }
    }
}

#[derive(Resource)]
pub struct TilemapTextures {
    pub textures: TilemapTexture,
    pub material: Handle<TilemapMaterial>,
}

impl TilemapTextures {
    pub fn new(textures: TilemapTexture, material: Handle<TilemapMaterial>) -> Self {
        Self { textures, material }
    }

    pub fn get_atlas_index_tile(&self, tile: TileData) -> TileTextureIndex {
        let folder = if tile.is_wall() { "walls" } else { "floors" };
        self.get_atlas_index(folder, tile.filename())
    }

    pub fn get_atlas_index_object(&self, object: ObjectData) -> TileTextureIndex {
        self.get_atlas_index("objects", object.filename())
    }

    fn get_atlas_index(&self, folder: &'static str, filename: &'static str) -> TileTextureIndex {
        TileTextureIndex(match &self.textures {
            TilemapTexture::Vector(textures) => textures
                .iter()
                .position(|t| {
                    let paths_string = t.path().unwrap().to_string();

                    paths_string.ends_with(format!("{folder}/{filename}.png").as_str())
                        || paths_string.ends_with(format!("{folder}\\{filename}.png").as_str())
                })
                .unwrap() as u32,

            _ => 0,
        })
    }
}
