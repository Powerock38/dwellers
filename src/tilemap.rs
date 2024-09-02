use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::{
    map::{
        TilemapGridSize, TilemapId, TilemapRenderSettings, TilemapSize, TilemapTexture,
        TilemapTileSize,
    },
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};

use crate::{utils::div_to_floor, ObjectData, TileData, TilePlaced};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
pub const CHUNK_SIZE: u32 = 64;
const RENDER_CHUNK_SIZE: u32 = CHUNK_SIZE * 2;

#[derive(Resource, Default)]
pub struct TilemapData {
    pub chunks: HashMap<IVec2, Vec<Option<TilePlaced>>>,
    pub tiles_to_spawn: Vec<(IVec2, TilePlaced)>,
    pub chunks_to_remove: Vec<IVec2>,
}

impl TilemapData {
    #[inline]
    pub fn index_to_chunk(index: IVec2) -> (IVec2, usize) {
        let isize = IVec2::splat(CHUNK_SIZE as i32);
        let c = div_to_floor(index, isize);
        let idx = index - c * isize;
        (c, (idx.y * isize.x + idx.x) as usize)
    }

    #[inline]
    pub fn chunk_to_index(chunk_index: IVec2, local_index: usize) -> IVec2 {
        chunk_index * CHUNK_SIZE as i32
            + IVec2::new(
                local_index as i32 % CHUNK_SIZE as i32,
                local_index as i32 / CHUNK_SIZE as i32,
            )
    }

    pub fn set(&mut self, index: IVec2, tile: TilePlaced) {
        self.tiles_to_spawn.push((index, tile));

        let idx = Self::index_to_chunk(index);
        self.chunks
            .entry(idx.0)
            .or_insert_with(|| vec![None; (CHUNK_SIZE * CHUNK_SIZE) as usize])[idx.1] = Some(tile);
    }

    pub fn get(&self, index: IVec2) -> Option<TilePlaced> {
        let idx = Self::index_to_chunk(index);
        self.chunks
            .get(&idx.0)
            .and_then(|c| c.get(idx.1))
            .copied()
            .flatten()
    }

    pub fn set_chunk(&mut self, chunk_index: IVec2, chunk_data: Vec<TilePlaced>) {
        self.tiles_to_spawn.extend(
            chunk_data
                .iter()
                .enumerate()
                .map(|(i, tile)| (Self::chunk_to_index(chunk_index, i), *tile)),
        );

        self.chunks
            .insert(chunk_index, chunk_data.into_iter().map(Some).collect());
    }

    pub fn remove_chunk(&mut self, index: IVec2) -> Option<Vec<Option<TilePlaced>>> {
        self.chunks_to_remove.push(index);
        self.chunks.remove(&index)
    }

    pub fn local_index_to_global(chunk_index: IVec2, local_index: IVec2) -> IVec2 {
        chunk_index * CHUNK_SIZE as i32 + local_index
    }

    pub fn neighbours(&self, pos: IVec2) -> Vec<(IVec2, TilePlaced)> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let index = pos + p;

                if let Some(tile) = self.get(index) {
                    return Some((index, tile));
                }

                None
            })
            .collect()
    }

    pub fn non_blocking_neighbours_pos(&self, pos: IVec2, diagonal: bool) -> Vec<IVec2> {
        let mut result: Vec<IVec2> = self
            .neighbours(pos)
            .into_iter()
            .filter_map(|(index, tile)| {
                if tile.is_blocking() {
                    None
                } else {
                    Some(index)
                }
            })
            .collect();

        if diagonal {
            let diagonal_directions = [
                IVec2::new(1, 1),
                IVec2::new(-1, 1),
                IVec2::new(1, -1),
                IVec2::new(-1, -1),
            ];

            for diag_pos in diagonal_directions {
                let diag_index = pos + diag_pos;

                if let Some(diag_tile) = self.get(diag_index) {
                    if !diag_tile.is_blocking() {
                        let adj_blocking = [IVec2::new(diag_pos.x, 0), IVec2::new(0, diag_pos.y)]
                            .into_iter()
                            .any(|adj| {
                                self.get(pos + adj).map_or(true, |t| t.id.data().is_wall())
                                // Do not allow diagonal movement if there is a wall, but allow if it's a blocking object
                            });

                        if !adj_blocking {
                            result.push(diag_index);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn find_from_center(index: IVec2, is_valid: impl Fn(IVec2) -> bool) -> Option<IVec2> {
        for radius in 0..=(CHUNK_SIZE as i32 / 2) {
            // Top and Bottom edges of the square
            for x in (index.x - radius).max(1)..=(index.x + radius).min(CHUNK_SIZE as i32 - 2) {
                // Top edge
                let top_y = (index.y - radius).max(1);
                if is_valid(IVec2::new(x, top_y)) {
                    return Some(IVec2::new(x, top_y));
                }
                // Bottom edge
                let bottom_y = (index.y + radius).min(CHUNK_SIZE as i32 - 2);
                if is_valid(IVec2::new(x, bottom_y)) {
                    return Some(IVec2::new(x, bottom_y));
                }
            }

            // Left and Right edges of the square (excluding corners already checked)
            for y in ((index.y - radius + 1).max(1))
                ..=((index.y + radius - 1).min(CHUNK_SIZE as i32 - 2))
            {
                // Left edge
                let left_x = (index.x - radius).max(1);
                if is_valid(IVec2::new(left_x, y)) {
                    return Some(IVec2::new(left_x, y));
                }
                // Right edge
                let right_x = (index.x + radius).min(CHUNK_SIZE as i32 - 2);
                if is_valid(IVec2::new(right_x, y)) {
                    return Some(IVec2::new(right_x, y));
                }
            }
        }

        None
    }
}

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
            new_tilemap(
                tilemap_textures.textures.clone(),
                tile_storage,
                pos.extend(0.0),
            ),
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
}

impl TilemapTextures {
    pub fn new(textures: TilemapTexture) -> Self {
        Self { textures }
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
                    t.path()
                        .unwrap()
                        .to_string()
                        .ends_with(format!("{folder}/{filename}.png").as_str())
                })
                .unwrap_or(0) as u32,

            _ => 0,
        })
    }
}
