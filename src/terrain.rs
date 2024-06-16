use bevy::prelude::*;
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use noise::{NoiseFn, Perlin, RidgedMulti};

use crate::{
    standard_tilemap_bundle,
    tiles::{ObjectData, TileData},
    TilemapData,
};

pub const TERRAIN_SIZE: u32 = 256;

const TREE_NOISE_SCALE: f64 = 10.0;
const MOUNTAIN_NOISE_SCALE: f64 = 5.0;

pub fn spawn_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn((Camera2dBundle::default(),));

    let entity = commands.spawn_empty().id();

    let mut tilemap = standard_tilemap_bundle(entity, asset_server, materials, textures);

    let base_tile = TileData::GRASS_FLOOR;

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(TERRAIN_SIZE)),
        base_tile.tile_builder(),
    );

    let mut tilemap_data = TilemapData::new(base_tile, TERRAIN_SIZE as usize);

    let seed = rand::random();

    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            let index = IVec2::new(x as i32, y as i32);
            let u = x as f64 / TERRAIN_SIZE as f64;
            let v = y as f64 / TERRAIN_SIZE as f64;

            let noise = RidgedMulti::<Perlin>::new(seed);

            // Mountains
            let mountain_noise_value =
                noise.get([u * MOUNTAIN_NOISE_SCALE, v * MOUNTAIN_NOISE_SCALE]);
            if mountain_noise_value < -0.1 {
                let tile = if mountain_noise_value < -0.3 {
                    TileData::STONE_WALL
                } else {
                    TileData::DIRT_WALL
                };

                tile.set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );

                continue;
            }

            // Rivers
            if mountain_noise_value > 0.5 {
                TileData::WATER.set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );

                continue;
            }

            // Trees
            let tree_noise_value = noise.get([u * TREE_NOISE_SCALE, v * TREE_NOISE_SCALE]);
            if tree_noise_value > 0.0 {
                // 0.5 {
                TileData::GRASS_FLOOR.with(ObjectData::TREE).set_at(
                    index,
                    &mut commands,
                    &mut tilemap.storage,
                    &mut tilemap_data,
                );
            }
        }
    }

    commands.entity(entity).insert((tilemap, tilemap_data));
}
