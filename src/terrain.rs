use std::io::Write;

use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use noise::{NoiseFn, Perlin, RidgedMulti};
use rand::Rng;

use crate::{
    data::ObjectId,
    extract_ok, init_tilemap,
    tiles::{TileData, TileKind},
    SaveName, TilemapData, CHUNK_SIZE, SAVE_DIR,
};

//FIXME: Terrain generation repeats itself
const TREE_NOISE_SCALE: f64 = 1.0 / 32.0;
const MOUNTAIN_NOISE_SCALE: f64 = 1.0 / 128.0;

#[derive(Event)]
pub struct LoadChunk(pub IVec2);

#[derive(Event)]
pub struct UnloadChunk(pub IVec2);

#[derive(Event)]
pub struct SpawnEntitiesOnChunk(pub IVec2);

pub fn spawn_new_terrain(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_spawn_entities_on_chunk: EventWriter<SpawnEntitiesOnChunk>,
    save_name: Res<SaveName>,
) {
    init_tilemap(commands, asset_server, materials, textures);

    let save_folder = format!("assets/{SAVE_DIR}/{}", save_name.0);
    std::fs::create_dir(save_folder).expect("Error while creating save folder");

    ev_load_chunk.send(LoadChunk(IVec2::ZERO));
    ev_spawn_entities_on_chunk.send(SpawnEntitiesOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut ev_load: EventReader<LoadChunk>,
    mut ev_unload: EventReader<UnloadChunk>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
    save_name: Res<SaveName>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    // Seed is based on the save name
    let seed = save_name.0.as_bytes().iter().map(|b| *b as u32).sum();
    let noise = RidgedMulti::<Perlin>::new(seed);

    let save_folder = format!("assets/{SAVE_DIR}/{}", save_name.0);

    let mut loaded = vec![];

    for LoadChunk(chunk_index) in ev_load.read() {
        if loaded.contains(chunk_index) {
            continue;
        }

        loaded.push(*chunk_index);

        if tilemap_data.data.chunks.contains_key(chunk_index) {
            continue;
        }

        // Try to load the chunk from the save
        if let Some(chunk_data) = std::fs::read(format!(
            "{save_folder}/{}_{}.bin",
            chunk_index.x, chunk_index.y
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

                    let u = index.x as f64;
                    let v = index.y as f64;

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
    }

    for UnloadChunk(chunk_index) in ev_unload.read() {
        let Some(chunk) = tilemap_data.data.chunks.get(chunk_index) else {
            continue;
        };

        debug!("Unloading chunk {}", chunk_index);

        let chunk = chunk.iter().filter_map(|t| *t).collect::<Vec<_>>();

        let chunk_encoded = bitcode::encode(&chunk);

        let save_folder = save_folder.clone();
        let x = chunk_index.x;
        let y = chunk_index.y;

        IoTaskPool::get()
            .spawn(async move {
                std::fs::File::create(format!("{save_folder}/{x}_{y}.bin"))
                    .and_then(|mut file| file.write(chunk_encoded.as_slice()))
                    .expect("Error while writing terrain to file");
            })
            .detach();
        tilemap_data.data.remove_chunk(*chunk_index);
        tilemap.remove_chunk(&mut commands, *chunk_index);
    }
}

pub fn update_terrain(
    mut commands: Commands,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    let chunks = &tilemap_data.data.chunks.clone(); // FIXME: clone

    for (chunk_index, _chunk) in chunks {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let index = TilemapData::local_index_to_global(
                    *chunk_index,
                    IVec2::new(x as i32, y as i32),
                );

                if let Some(tile) = tilemap_data.get(index) {
                    match tile.kind {
                        TileKind::Floor(Some(ObjectId::Farm)) => {
                            let mut rng = rand::thread_rng();

                            if rng.gen_bool(0.01) {
                                tile.with(ObjectId::WheatPlant).set_at(
                                    index,
                                    &mut commands,
                                    &mut tilemap,
                                    &mut tilemap_data,
                                );
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
