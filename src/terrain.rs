use bevy::prelude::*;
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use noise::{NoiseFn, Perlin, RidgedMulti};

use crate::{
    data::ObjectId, extract_ok, init_tilemap, tiles::TileData, SaveName, TilemapData, CHUNK_SIZE,
    SAVE_DIR,
};

//FIXME: Terrain generation repeats itself
const TREE_NOISE_SCALE: f64 = 1.0 / 128.0;
const MOUNTAIN_NOISE_SCALE: f64 = 1.0 / 128.0;

#[derive(Event)]
pub struct LoadChunk(pub IVec2);

#[derive(Event)]
pub struct SpawnEntitiesOnChunk(pub IVec2);

pub fn spawn_new_terrain(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_spawn_entities_on_chunk: EventWriter<SpawnEntitiesOnChunk>,
) {
    init_tilemap(commands, asset_server, materials, textures);
    ev_load_chunk.send(LoadChunk(IVec2::ZERO));
    ev_spawn_entities_on_chunk.send(SpawnEntitiesOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut events: EventReader<LoadChunk>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
    save_name: Res<SaveName>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    // Seed is based on the save name
    let seed = save_name.0.as_bytes().iter().map(|b| *b as u32).sum();
    let noise = RidgedMulti::<Perlin>::new(seed);

    let mut loaded = vec![];

    for LoadChunk(chunk_index) in events.read() {
        if loaded.contains(chunk_index) || tilemap_data.data.chunks.contains_key(chunk_index) {
            continue;
        }

        // Try to load the chunk from the save
        if let Some(chunk_data) = std::fs::read(format!(
            "assets/{SAVE_DIR}/{}/{}_{}.bin",
            save_name.0, chunk_index.x, chunk_index.y
        ))
        .ok()
        .and_then(|data| bitcode::decode::<Vec<TileData>>(&data).ok())
        {
            debug!("Loading chunk {} from save file", chunk_index);

            // Load in TilemapData
            tilemap_data.set_chunk(*chunk_index, chunk_data);

            // Load in TilemapStorage
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let index = TilemapData::local_index_to_global(
                        *chunk_index,
                        IVec2::new(x as i32, y as i32),
                    );

                    if let Some(tile_data) = tilemap_data.get(index) {
                        tilemap.set(&mut commands, index, tile_data.tile_builder());

                        TileData::update_light_level(
                            index,
                            &mut commands,
                            &mut tilemap,
                            &tilemap_data,
                        );
                    }
                }
            }
            //TODO update light level of surrounding chunks?
        } else {
            // If the chunk is not in the save, generate it

            debug!("Generating chunk {}", chunk_index);

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let index = TilemapData::local_index_to_global(
                        *chunk_index,
                        IVec2::new(x as i32, y as i32),
                    );

                    let u = x as f64;
                    let v = y as f64;

                    // Mountains
                    let mountain_noise_value =
                        noise.get([u * MOUNTAIN_NOISE_SCALE, v * MOUNTAIN_NOISE_SCALE]);
                    if mountain_noise_value < -0.1 {
                        let tile = if mountain_noise_value < -0.3 {
                            TileData::STONE_WALL
                        } else {
                            TileData::DIRT_WALL
                        };

                        tile.set_at(index, &mut commands, &mut tilemap, &mut tilemap_data);

                        continue;
                    }

                    // Rivers
                    if mountain_noise_value > 0.5 {
                        TileData::WATER.set_at(
                            index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        continue;
                    }

                    // Vegetation
                    let vegetation_noise_value =
                        noise.get([u * TREE_NOISE_SCALE, v * TREE_NOISE_SCALE]);
                    if vegetation_noise_value > 0.0 {
                        let vegetation = if vegetation_noise_value > 0.7 {
                            ObjectId::TallGrass
                        } else {
                            ObjectId::Tree
                        };

                        TileData::GRASS_FLOOR.with(vegetation).set_at(
                            index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        continue;
                    }

                    // Base tile
                    TileData::GRASS_FLOOR.set_at(
                        index,
                        &mut commands,
                        &mut tilemap,
                        &mut tilemap_data,
                    );
                }
            }
        }

        loaded.push(*chunk_index);
    }
}

// FIXME: doesnt work because of borrow checker
// TODO: loop over tiles individually (take advantage of ECS)
// pub fn update_loaded_chunks(
//     mut commands: Commands,
//     mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
// ) {
//     let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

//     let chunks = &tilemap_data.data.chunks;

//     for (chunk_index, _chunk) in chunks {
//         for x in 0..CHUNK_SIZE {
//             for y in 0..CHUNK_SIZE {
//                 let index = TilemapData::local_index_to_global(
//                        *chunk_index,
//                        IVec2::new(x as i32, y as i32),
//                    );

//                 if let Some(tile) = tilemap_data.get(index) {
//                     match tile.kind {
//                         TileKind::Floor(Some(ObjectId::Farm)) => {
//                             let mut rng = rand::thread_rng();

//                             if rng.gen_bool(0.01) {
//                                 tile.with(ObjectId::WheatPlant).set_at(
//                                     index,
//                                     &mut commands,
//                                     &mut tilemap,
//                                     &mut tilemap_data,
//                                 );
//                             }
//                         }

//                         _ => {}
//                     }
//                 }
//             }
//         }
//     }
// }
