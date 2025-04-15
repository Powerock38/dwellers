use bevy::prelude::*;
use noise::{
    core::worley::distance_functions::euclidean_squared, Abs, Billow, Fbm, NoiseFn, OpenSimplex,
    Perlin, Simplex, Worley,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    data::{ObjectId, StructureId, TileId},
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    tilemap_data::TilemapData,
    tiles::TilePlaced,
    MobBundle, SpawnMobsOnChunk, CHUNK_SIZE,
};

const MOBS_SCALE: f64 = 0.1;
const MOBS_THRESHOLD: f64 = 0.0;

const CLIMATE_SCALE: f64 = 0.01;
const DESERT_THRESHOLD: f64 = 0.5;

const STRUCTURES_SCALE: f64 = 0.2;

const MOUNTAINS_SCALE: f64 = 0.004;
const MOUNTAINS_DIRT_THRESHOLD: f64 = -0.3;
const MOUNTAINS_STONE_THRESHOLD: f64 = -0.2;

const MOUNTAINS_CAVES_THRESHOLD: f64 = 0.0;
const CAVES_TUNNELS_SCALE: f64 = 0.05;
const CAVES_TUNNELS_THRESHOLD: f64 = 0.05;
const CAVES_ROOMS_SCALE: f64 = 0.1;
const CAVES_ROOMS_THRESHOLD: f64 = 0.9;

const MOUNTAINS_LAVA_THRESHOLD: f64 = 0.5;
const LAVA_SCALE: f64 = 0.1;
const LAVA_THRESHOLD: f64 = 0.0;

const ORES_SCALE: f64 = 0.2;
const ORES_THRESHOLD: f64 = 0.7;

const RIVER_DEEP_THRESHOLD: f64 = 0.75;
const RIVER_THRESHOLD: f64 = 0.7;
const RIVER_SHORE_THRESHOLD: f64 = 0.65;

const VEGETATION_ZONES_SCALE: f64 = 0.05;
const VEGETATION_ZONES_THRESHOLD: f64 = 0.4;

const VEGETATION_SCALE: f64 = 0.5;
const TREE_THRESHOLD: f64 = 0.4;
const PLANT_THRESHOLD: f64 = 0.7;

pub fn generate_terrain(commands: &mut Commands, seed: u32, chunk_index: IVec2) -> Vec<TilePlaced> {
    let noise_mountains = Billow::<Perlin>::new(seed);
    let noise_climate = Simplex::new(seed);
    let noise_structures = Simplex::new(seed + 1);
    let noise_ores = Perlin::new(seed);
    let noise_vegetation = Worley::new(seed);
    let noise_vegetation_zones = Perlin::new(seed + 1);
    let noise_caves_tunnels = Abs::new(OpenSimplex::new(seed));
    let noise_caves_rooms = Worley::new(seed + 1).set_distance_function(euclidean_squared);
    let noise_lava = Fbm::<Perlin>::new(seed);

    // Generate mobs
    if noise_climate.get([
        chunk_index.x as f64 * MOBS_SCALE,
        chunk_index.y as f64 * MOBS_SCALE,
    ]) > MOBS_THRESHOLD
    {
        commands.send_event(SpawnMobsOnChunk(chunk_index));
    }

    // Generate terrain
    let mut tiles = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize);

    let mut mountainy_count = 0;
    let mut plainy_count = 0;

    // Generate tiles
    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let index =
                TilemapData::local_index_to_global(chunk_index, IVec2::new(x as i32, y as i32));

            let u = index.x as f64;
            let v = index.y as f64;

            let climate_noise_value = noise_climate.get([u * CLIMATE_SCALE, v * CLIMATE_SCALE]);

            // Mountains
            let mountain_noise_value =
                noise_mountains.get([u * MOUNTAINS_SCALE, v * MOUNTAINS_SCALE]);

            // Ore
            let ores_noise_value = noise_ores.get([u * ORES_SCALE, v * ORES_SCALE]);

            if mountain_noise_value > MOUNTAINS_DIRT_THRESHOLD {
                let mut tile = TileId::DirtWall.place();

                if mountain_noise_value > MOUNTAINS_STONE_THRESHOLD {
                    if ores_noise_value > ORES_THRESHOLD {
                        tile = TileId::StoneWall.with(ObjectId::CopperOre);
                    } else {
                        tile = TileId::StoneWall.place();
                    }

                    // Caves
                    let tunnels_noise_value =
                        noise_caves_tunnels.get([u * CAVES_TUNNELS_SCALE, v * CAVES_TUNNELS_SCALE]);
                    let rooms_noise_value =
                        noise_caves_rooms.get([u * CAVES_ROOMS_SCALE, v * CAVES_ROOMS_SCALE]);

                    if mountain_noise_value > MOUNTAINS_CAVES_THRESHOLD
                        && (tunnels_noise_value < CAVES_TUNNELS_THRESHOLD
                            || rooms_noise_value > CAVES_ROOMS_THRESHOLD)
                    {
                        tile = TileId::StoneFloor.place();
                    }
                }

                // Lava
                if mountain_noise_value > MOUNTAINS_LAVA_THRESHOLD {
                    let lava_noise_value = noise_lava.get([u * LAVA_SCALE, v * LAVA_SCALE]);
                    if lava_noise_value > LAVA_THRESHOLD {
                        tile = TileId::Lava.place();
                    }
                }

                tiles.push(tile);
                mountainy_count += 1;

                continue;
            }

            // Rivers
            if mountain_noise_value < -RIVER_DEEP_THRESHOLD {
                if ores_noise_value > ORES_THRESHOLD {
                    tiles.push(TileId::Water.with(ObjectId::FishingSpot));
                } else {
                    tiles.push(TileId::Water.place());
                }
                continue;
            }

            if mountain_noise_value < -RIVER_THRESHOLD {
                tiles.push(TileId::ShallowWater.place());
                continue;
            }

            // River shores
            if mountain_noise_value < -RIVER_SHORE_THRESHOLD {
                tiles.push(TileId::SandFloor.place());
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

            tiles.push(ground_tile);
            plainy_count += 1;
        }
    }

    let mountainy = mountainy_count as f32 / (CHUNK_SIZE * CHUNK_SIZE) as f32 > 0.5;
    let plainy = plainy_count as f32 / (CHUNK_SIZE * CHUNK_SIZE) as f32 > 0.5;

    // Generate structures
    let structure_noise_value = noise_structures.get([
        chunk_index.x as f64 * STRUCTURES_SCALE,
        chunk_index.y as f64 * STRUCTURES_SCALE,
    ]);

    let mut rng: StdRng =
        SeedableRng::seed_from_u64((seed as i32 + chunk_index.x + chunk_index.y) as u64);

    let mut structure = match structure_noise_value {
        0.0..=0.5 if mountainy => StructureId::DungeonCircleRoom,
        0.5..1.0 if plainy => StructureId::Outpost,
        _ => return tiles,
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

    let structure_pos = TilemapData::local_index_to_global(chunk_index, structure_local_pos);

    for x in 0..structure.x_size() {
        for y in 0..structure.y_size() {
            if let Some(tile) = structure.get_tile(x, y) {
                let index = structure_pos + IVec2::new(x as i32, y as i32);
                let (_, i) = TilemapData::index_to_chunk(index);
                tiles[i] = *tile;
            }
        }
    }

    // Add mobs to the structure
    for (pos, mob) in structure.mobs() {
        let index = structure_pos + pos.as_ivec2();
        commands.spawn(MobBundle::new(*mob, index));
    }

    tiles
}

pub fn update_terrain(
    mut commands: Commands,
    mut tilemap_data: ResMut<TilemapData>,
    q_tasks: Query<&Task>,
) {
    let mut to_set = vec![]; //because cant modify tilemap_data while iterating

    let mut rng = rand::rng();

    for (chunk_index, chunk) in &tilemap_data.chunks {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let index = TilemapData::local_index_to_global(
                    *chunk_index,
                    IVec2::new(x as i32, y as i32),
                );

                let (_, i) = TilemapData::index_to_chunk(index);
                let tile = chunk[i];

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
                            if let Some(index) = TilemapData::find_from_center(index, 4, |index| {
                                if let Some(TilePlaced {
                                    object: Some(ObjectId::WheatPlant),
                                    ..
                                }) = tilemap_data.get(index)
                                {
                                    return !q_tasks.iter().any(|task| task.pos == index);
                                }

                                false
                            }) {
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

    for (index, tile) in to_set {
        tilemap_data.set(index, tile);
    }
}
