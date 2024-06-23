use std::{fs::File, io::Write};

use bevy::{
    prelude::*,
    render::camera::{CameraMainTextureUsages, CameraRenderGraph},
    tasks::IoTaskPool,
};
use bevy_entitiles::{
    render::material::StandardTilemapMaterial,
    tilemap::{map::TilemapTextures, tile::Tile},
};

use crate::{init_tilemap, tilemap::TilemapData, Dweller, GameState, Mob, Task};

pub const SAVE_DIR: &str = "saves";

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SaveName(pub String);

#[derive(Resource)]
pub struct SaveGame(pub String);

#[derive(Resource)]
pub struct LoadGame(pub String);

pub fn save_world(
    mut commands: Commands,
    save_game: Option<Res<SaveGame>>,
    q_tilemap_data: Query<&TilemapData>,
    q_dwellers: Query<Entity, With<Dweller>>,
    q_tasks: Query<Entity, With<Task>>,
    q_mobs: Query<Entity, With<Mob>>,
    world: &World,
) {
    if let Some(save_game) = save_game {
        if save_game.is_added() {
            commands.remove_resource::<SaveGame>();
            info!("Saving scene: {}", save_game.0);

            let tilemap_data = q_tilemap_data.single();

            // Save entities with bevy reflection

            let app_type_registry = world.resource::<AppTypeRegistry>().clone();

            let scene = DynamicSceneBuilder::from_world(world)
                .deny_all_resources()
                .allow_all()
                .allow_resource::<SaveName>()
                .deny::<CameraRenderGraph>()
                .deny::<CameraMainTextureUsages>()
                .deny::<Handle<Image>>()
                .deny::<Sprite>()
                .extract_resources()
                .extract_entities(q_dwellers.iter())
                .extract_entities(q_tasks.iter())
                .extract_entities(q_mobs.iter())
                .remove_empty_entities()
                .build();

            match scene.serialize_ron(&app_type_registry) {
                Ok(serialized) => {
                    let save_name = save_game.0.clone();

                    let save_folder = format!("assets/{SAVE_DIR}/{save_name}");

                    std::fs::create_dir(save_folder.clone())
                        .expect("Error while creating save folder");

                    // Save tilemap with bitcode
                    for (chunk_index, chunk) in &tilemap_data.data.chunks {
                        let save_folder = save_folder.clone();
                        let chunk_name = format!("{}_{}", chunk_index.x, chunk_index.y);

                        let chunk = chunk.iter().filter_map(|t| *t).collect::<Vec<_>>();

                        let chunk_encoded = bitcode::encode(&chunk);

                        IoTaskPool::get()
                            .spawn(async move {
                                File::create(format!("{save_folder}/{chunk_name}.bin"))
                                    .and_then(|mut file| file.write(chunk_encoded.as_slice()))
                                    .expect("Error while writing terrain to file");
                            })
                            .detach();
                    }

                    // Save tasks & entities with Bevy reflection
                    IoTaskPool::get()
                        .spawn(async move {
                            File::create(format!("{save_folder}/entities.ron"))
                                .and_then(|mut file| file.write(serialized.as_bytes()))
                                .expect("Error while writing entities to file");
                        })
                        .detach();
                }
                Err(e) => {
                    error!("Error while serializing the scene: {e:?}");
                }
            }
        }
    }
}

pub fn load_world(
    mut commands: Commands,
    load_game: Option<Res<LoadGame>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
    q_tilemap_data: Query<Entity, With<TilemapData>>,
    q_tiles: Query<Entity, With<Tile>>,
    q_dwellers: Query<Entity, With<Dweller>>,
    q_tasks: Query<Entity, With<Task>>,
    q_mobs: Query<Entity, With<Mob>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(load_game) = load_game {
        if load_game.is_added() {
            commands.remove_resource::<LoadGame>();
            info!("Loading scene: {}", load_game.0);

            // Despawn current scene
            if let Some(tilemap_data) = q_tilemap_data.iter().next() {
                commands.entity(tilemap_data).despawn_recursive();
            }

            for tile in q_tiles.iter() {
                commands.entity(tile).despawn_recursive();
            }

            for dweller in q_dwellers.iter() {
                commands.entity(dweller).despawn_recursive();
            }

            for task in q_tasks.iter() {
                commands.entity(task).despawn_recursive();
            }

            for mob in q_mobs.iter() {
                commands.entity(mob).despawn_recursive();
            }

            // Spawn new scene
            scene_spawner.spawn_dynamic(
                asset_server.load(format!("{SAVE_DIR}/{}/entities.ron", load_game.0.clone())),
            );

            // Init tilemap, chunks will be loaded from disk
            init_tilemap(commands, asset_server, materials, textures);

            next_state.set(GameState::Running);
        }
    }
}
