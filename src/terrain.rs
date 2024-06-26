use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use noise::{NoiseFn, Perlin, RidgedMulti, Simplex, Worley};
use rand::Rng;

use crate::{
    data::{ObjectId, TileId},
    extract_ok, init_tilemap,
    tiles::TilePlaced,
    SaveName, TilemapData, CHUNK_SIZE, SAVE_DIR,
};

const CLIMATE_NOISE_SCALE: f64 = 0.01;
const MOUNTAIN_NOISE_SCALE: f64 = 0.004;
const ORES_NOISE_SCALE: f64 = 0.1;
const VEGETATION_NOISE_SCALE: f64 = 0.5;
const VEGETATION_ZONES_NOISE_SCALE: f64 = 0.05;

#[derive(Event)]
pub struct LoadChunk(pub IVec2);

#[derive(Event)]
pub struct UnloadChunk(pub IVec2);

#[derive(Event)]
pub struct SpawnDwellersOnChunk(pub IVec2);

#[derive(Event)]
pub struct SpawnMobsOnChunk(pub IVec2);

pub fn spawn_new_terrain(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_spawn_dwellers: EventWriter<SpawnDwellersOnChunk>,
    save_name: Res<SaveName>,
) {
    init_tilemap(commands, asset_server, materials, textures);

    let save_folder = format!("assets/{SAVE_DIR}/{}", save_name.0);
    std::fs::create_dir(save_folder).expect("Error while creating save folder");

    ev_load_chunk.send(LoadChunk(IVec2::ZERO));
    ev_spawn_dwellers.send(SpawnDwellersOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut ev_load: EventReader<LoadChunk>,
    mut ev_unload: EventReader<UnloadChunk>,
    mut q_tilemap: Query<(&mut TilemapStorage, &mut TilemapData)>,
    save_name: Res<SaveName>,
    mut ev_spawn_mobs: EventWriter<SpawnMobsOnChunk>,
) {
    let (mut tilemap, mut tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

    // Seed is based on the save name
    let seed = save_name.0.as_bytes().iter().map(|b| *b as u32).sum();
    let noise = RidgedMulti::<Simplex>::new(seed);
    let noise_climate = Simplex::new(seed);
    let noise_ores = Perlin::new(seed);
    let noise_vegetation = Worley::new(seed);
    let noise_vegetation_zones = Perlin::new(seed + 1);

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
        .and_then(|data| bitcode::decode::<Vec<TilePlaced>>(&data).ok())
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

                    if let Some(tile) = tilemap_data.get(index) {
                        tilemap.set(&mut commands, index, tile.tile_builder());

                        TilePlaced::update_light_level(
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

            if noise_climate.get([chunk_index.x as f64, chunk_index.y as f64]) > 0.7 {
                ev_spawn_mobs.send(SpawnMobsOnChunk(*chunk_index));
            }

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let index = TilemapData::local_index_to_global(
                        *chunk_index,
                        IVec2::new(x as i32, y as i32),
                    );

                    let u = index.x as f64;
                    let v = index.y as f64;

                    let climate_noise_value =
                        noise_climate.get([u * CLIMATE_NOISE_SCALE, v * CLIMATE_NOISE_SCALE]);

                    // Mountains
                    let mountain_noise_value =
                        noise.get([u * MOUNTAIN_NOISE_SCALE, v * MOUNTAIN_NOISE_SCALE]);

                    if mountain_noise_value < -0.4 {
                        let tile = if mountain_noise_value < -0.55 {
                            let ores_noise_value =
                                noise_ores.get([u * ORES_NOISE_SCALE, v * ORES_NOISE_SCALE]);

                            if ores_noise_value > 0.5 {
                                TileId::StoneWall.with(ObjectId::CopperOre)
                            } else {
                                TileId::StoneWall.empty()
                            }
                        } else {
                            TileId::DirtWall.empty()
                        };

                        tile.set_at(index, &mut commands, &mut tilemap, &mut tilemap_data);

                        continue;
                    }

                    // Rivers
                    if mountain_noise_value > 0.5 {
                        TileId::Water.empty().set_at(
                            index,
                            &mut commands,
                            &mut tilemap,
                            &mut tilemap_data,
                        );

                        continue;
                    }

                    // Vegetation
                    let vegetation_noise_value = noise_vegetation
                        .get([u * VEGETATION_NOISE_SCALE, v * VEGETATION_NOISE_SCALE]);

                    let vegetation_zones_noise_value = noise_vegetation_zones.get([
                        u * VEGETATION_ZONES_NOISE_SCALE,
                        v * VEGETATION_ZONES_NOISE_SCALE,
                    ]);

                    let vegetation = if vegetation_zones_noise_value > 0.4 {
                        if vegetation_noise_value > 0.4 {
                            if climate_noise_value > 0.5 {
                                Some(ObjectId::PalmTree)
                            } else {
                                Some(ObjectId::Tree)
                            }
                        } else if vegetation_noise_value < -0.7 {
                            if climate_noise_value > 0.5 {
                                Some(ObjectId::Cactus)
                            } else {
                                Some(ObjectId::TallGrass)
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut ground_tile = if climate_noise_value > 0.5 {
                        TileId::SandFloor.empty()
                    } else {
                        TileId::GrassFloor.empty()
                    };

                    if let Some(object) = vegetation {
                        ground_tile = ground_tile.id.with(object);
                    }

                    ground_tile.set_at(index, &mut commands, &mut tilemap, &mut tilemap_data);
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
                std::fs::write(format!("{save_folder}/{x}_{y}.bin"), chunk_encoded)
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
                    match tile.object {
                        Some(ObjectId::Farm) => {
                            let mut rng = rand::thread_rng();

                            if rng.gen_bool(0.01) {
                                tile.id.with(ObjectId::WheatPlant).set_at(
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
