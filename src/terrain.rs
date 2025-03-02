use bevy::{prelude::*, tasks::IoTaskPool};
use noise::{NoiseFn, Perlin, RidgedMulti, Simplex, Worley};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    data::{ObjectId, StructureId, TileId},
    init_tilemap,
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    tilemap_data::TilemapData,
    tiles::TilePlaced,
    utils::write_to_file,
    MobBundle, SaveName, SpawnDwellersOnChunk, SpawnMobsOnChunk, CHUNK_SIZE, SAVE_DIR,
};

const MOBS_SCALE: f64 = 0.1;
const MOBS_THRESHOLD: f64 = 0.0;

const CLIMATE_SCALE: f64 = 0.01;
const DESERT_THRESHOLD: f64 = 0.5;

const STRUCTURES_SCALE: f64 = 0.2;

const MOUNTAINS_SCALE: f64 = 0.004;
const MOUNTAINS_DIRT_THRESHOLD: f64 = 0.2;
const MOUNTAINS_STONE_THRESHOLD: f64 = 0.3;
const RIVER_SHORE_THRESHOLD: f64 = 0.45;
const RIVER_THRESHOLD: f64 = 0.5;

const ORES_SCALE: f64 = 0.2;
const ORES_THRESHOLD: f64 = 0.7;

const VEGETATION_ZONES_SCALE: f64 = 0.05;
const VEGETATION_ZONES_THRESHOLD: f64 = 0.4;

const VEGETATION_SCALE: f64 = 0.5;
const TREE_THRESHOLD: f64 = 0.4;
const PLANT_THRESHOLD: f64 = 0.7;

#[derive(Event)]
pub struct LoadChunk(pub IVec2);

#[derive(Event)]
pub struct UnloadChunk(pub IVec2);

pub fn spawn_new_terrain(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_spawn_dwellers: EventWriter<SpawnDwellersOnChunk>,
) {
    init_tilemap(commands, asset_server);

    ev_load_chunk.send(LoadChunk(IVec2::ZERO));
    ev_spawn_dwellers.send(SpawnDwellersOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut ev_load: EventReader<LoadChunk>,
    mut ev_unload: EventReader<UnloadChunk>,
    mut tilemap_data: ResMut<TilemapData>,
    save_name: Res<SaveName>,
    mut ev_spawn_mobs: EventWriter<SpawnMobsOnChunk>,
) {
    // Seed is based on the save name
    let seed = save_name.0.as_bytes().iter().map(|b| *b as u32).sum();
    let noise_mountains = RidgedMulti::<Perlin>::new(seed);
    let noise_climate = Simplex::new(seed);
    let noise_structures = Simplex::new(seed + 1);
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

        if tilemap_data.chunks.contains_key(chunk_index) {
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
        } else {
            // If the chunk is not in the save, generate it

            debug!("Generating chunk {}", chunk_index);

            // Generate mobs
            if noise_climate.get([
                chunk_index.x as f64 * MOBS_SCALE,
                chunk_index.y as f64 * MOBS_SCALE,
            ]) > MOBS_THRESHOLD
            {
                ev_spawn_mobs.send(SpawnMobsOnChunk(*chunk_index));
            }

            // Generate tiles
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let index = TilemapData::local_index_to_global(
                        *chunk_index,
                        IVec2::new(x as i32, y as i32),
                    );

                    let u = index.x as f64;
                    let v = index.y as f64;

                    let climate_noise_value =
                        noise_climate.get([u * CLIMATE_SCALE, v * CLIMATE_SCALE]);

                    // Mountains
                    let mountain_noise_value =
                        noise_mountains.get([u * MOUNTAINS_SCALE, v * MOUNTAINS_SCALE]);

                    if mountain_noise_value < -MOUNTAINS_DIRT_THRESHOLD {
                        let tile = if mountain_noise_value < -MOUNTAINS_STONE_THRESHOLD {
                            let ores_noise_value = noise_ores.get([u * ORES_SCALE, v * ORES_SCALE]);

                            if ores_noise_value > ORES_THRESHOLD {
                                TileId::StoneWall.with(ObjectId::CopperOre)
                            } else {
                                TileId::StoneWall.place()
                            }
                        } else {
                            TileId::DirtWall.place()
                        };

                        tilemap_data.set(index, tile);

                        continue;
                    }

                    // Rivers
                    if mountain_noise_value > RIVER_THRESHOLD {
                        tilemap_data.set(index, TileId::Water.place());

                        continue;
                    }

                    // River shores
                    if mountain_noise_value > RIVER_SHORE_THRESHOLD {
                        tilemap_data.set(index, TileId::SandFloor.place());

                        continue;
                    }

                    // Vegetation
                    let vegetation_noise_value =
                        noise_vegetation.get([u * VEGETATION_SCALE, v * VEGETATION_SCALE]);

                    let vegetation_zones_noise_value = noise_vegetation_zones
                        .get([u * VEGETATION_ZONES_SCALE, v * VEGETATION_ZONES_SCALE]);

                    let vegetation = if vegetation_zones_noise_value > VEGETATION_ZONES_THRESHOLD {
                        if vegetation_noise_value > TREE_THRESHOLD {
                            if climate_noise_value > DESERT_THRESHOLD {
                                Some(ObjectId::PalmTree)
                            } else {
                                Some(ObjectId::Tree)
                            }
                        } else if vegetation_noise_value < -PLANT_THRESHOLD {
                            if climate_noise_value > DESERT_THRESHOLD {
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

                    let mut ground_tile = if climate_noise_value > DESERT_THRESHOLD {
                        TileId::SandFloor.place()
                    } else {
                        TileId::GrassFloor.place()
                    };

                    if let Some(object) = vegetation {
                        ground_tile = ground_tile.id.with(object);
                    }

                    tilemap_data.set(index, ground_tile);
                }
            }

            // Generate structures
            let structure_noise_value = noise_structures.get([
                chunk_index.x as f64 * STRUCTURES_SCALE,
                chunk_index.y as f64 * STRUCTURES_SCALE,
            ]);

            let mut rng: StdRng =
                SeedableRng::seed_from_u64((seed as i32 + chunk_index.x + chunk_index.y) as u64);

            let mut structure = match structure_noise_value {
                0.0..=0.5 => StructureId::DungeonCircleRoom,
                0.5..1.0 => StructureId::Outpost,
                _ => continue,
            }
            .data();

            if rng.random_bool(0.5) {
                structure = structure.flip_horizontal();
            }

            if rng.random_bool(0.5) {
                structure = structure.flip_vertical();
            }

            let rotation = rng.random_range(0..=3);
            let clockwise = rng.random_bool(0.5);
            for _ in 0..rotation {
                structure = structure.rotate(clockwise);
            }

            let structure_size = structure.size().as_ivec2();

            let structure_local_pos = IVec2::new(
                rng.random_range(0..CHUNK_SIZE as i32 - structure_size.x),
                rng.random_range(0..CHUNK_SIZE as i32 - structure_size.y),
            );

            let structure_pos =
                TilemapData::local_index_to_global(*chunk_index, structure_local_pos);

            for x in 0..structure.x_size() {
                for y in 0..structure.y_size() {
                    if let Some(tile) = structure.get_tile(x, y) {
                        let index = structure_pos + IVec2::new(x as i32, y as i32);
                        tilemap_data.set(index, *tile);
                    }
                }
            }

            // Add mobs to the structure
            for (pos, mob) in structure.mobs() {
                let index = structure_pos + pos.as_ivec2();

                if let Some(tile) = tilemap_data.get(index) {
                    if tile.is_blocking() {
                        error!("Can't spawn mob on blocking tile {:?}", index);
                    } else {
                        commands.spawn(MobBundle::new(*mob, index));
                    }
                }
            }
        }
    }

    for UnloadChunk(chunk_index) in ev_unload.read() {
        let Some(chunk) = tilemap_data.chunks.get(chunk_index) else {
            continue;
        };

        debug!("Unloading chunk {}", chunk_index);

        let chunk_encoded = bitcode::encode(chunk);

        let save_folder = save_folder.clone();
        let x = chunk_index.x;
        let y = chunk_index.y;

        IoTaskPool::get()
            .spawn(async move {
                let path = format!("{save_folder}/{x}_{y}.bin");
                write_to_file(path, chunk_encoded);
            })
            .detach();
        tilemap_data.remove_chunk(*chunk_index);
    }
}

pub fn update_terrain(
    mut commands: Commands,
    mut tilemap_data: ResMut<TilemapData>,
    q_tasks: Query<&Task>,
) {
    let mut to_set = vec![]; //because cant modify tilemap_data while iterating

    let mut rng = rand::rng();

    for (chunk_index, _chunk) in &tilemap_data.chunks {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let index = TilemapData::local_index_to_global(
                    *chunk_index,
                    IVec2::new(x as i32, y as i32),
                );

                if let Some(tile) = tilemap_data.get(index) {
                    if let Some(object) = tile.object {
                        match object {
                            ObjectId::Farm => {
                                let chance = match tile.id {
                                    TileId::GrassFloor => 0.03,
                                    _ => 0.01,
                                };

                                if rng.random_bool(chance) {
                                    to_set.push((index, tile.id.with(ObjectId::WheatPlant)));
                                }
                            }

                            ObjectId::Scarecrow => {
                                if let Some(index) =
                                    TilemapData::find_from_center(index, 4, |index| {
                                        if let Some(TilePlaced {
                                            object: Some(ObjectId::WheatPlant),
                                            ..
                                        }) = tilemap_data.get(index)
                                        {
                                            return !q_tasks.iter().any(|task| task.pos == index);
                                        }

                                        false
                                    })
                                {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(index, TaskKind::Harvest, None, &tilemap_data),
                                        TaskNeeds::EmptyHands,
                                    ));
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }

    for (index, tile) in to_set {
        tilemap_data.set(index, tile);
    }
}
