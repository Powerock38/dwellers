use std::{fs::File, io::Write};

use bevy::{
    prelude::*,
    render::camera::{CameraMainTextureUsages, CameraRenderGraph},
    tasks::IoTaskPool,
};

use crate::{
    init_tilemap, tilemap_data::TilemapData, ChunkObjectLayer, ChunkTileLayer, Dweller, GameState,
    Mob, Task, TilemapMaterial, UnloadChunk,
};

pub const SAVE_DIR: &str = "saves";

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SaveName(pub String);

#[derive(Resource)]
pub struct SaveGame;

#[derive(Resource)]
pub struct LoadGame(pub String);

pub fn save_world_before(
    save_game: Option<Res<SaveGame>>,
    tilemap_data: Res<TilemapData>,
    mut ev_unload_w: EventWriter<UnloadChunk>,
) {
    if let Some(save_game) = save_game {
        if save_game.is_added() {
            for chunk_index in tilemap_data.chunks.keys() {
                ev_unload_w.send(UnloadChunk(*chunk_index));
            }
        }
    }
}

pub fn save_world(
    mut commands: Commands,
    save_name: Res<SaveName>,
    save_game: Option<Res<SaveGame>>,
    tilemap_data: Res<TilemapData>,
    q_dwellers: Query<Entity, With<Dweller>>,
    q_tasks: Query<Entity, With<Task>>,
    q_mobs: Query<Entity, With<Mob>>,
    world: &World,
) {
    if save_game.is_some() {
        info!("Saving scene: unloading all chunks...");

        if tilemap_data.chunks.is_empty() {
            commands.remove_resource::<SaveGame>();
            info!("Saving scene: serializing...");

            // Save entities with bevy reflection

            let app_type_registry = world.resource::<AppTypeRegistry>().clone();

            let scene = DynamicSceneBuilder::from_world(world)
                .deny_all_resources()
                .allow_all_components()
                .allow_resource::<SaveName>()
                .deny_component::<CameraRenderGraph>()
                .deny_component::<CameraMainTextureUsages>()
                .deny_component::<Sprite>()
                .extract_resources()
                .extract_entities(q_dwellers.iter())
                .extract_entities(q_tasks.iter())
                .extract_entities(q_mobs.iter())
                .remove_empty_entities()
                .build();

            let type_registry = app_type_registry.read();
            match scene.serialize(&type_registry) {
                Ok(serialized) => {
                    let save_folder = format!("assets/{SAVE_DIR}/{}", save_name.0);

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
    materials: ResMut<Assets<TilemapMaterial>>,
    q_chunks_tile_layer: Query<Entity, With<ChunkTileLayer>>,
    q_chunks_object_layer: Query<Entity, With<ChunkObjectLayer>>,
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
            for chunk_layer in q_chunks_tile_layer.iter() {
                commands.entity(chunk_layer).despawn_recursive();
            }

            for chunk_layer in q_chunks_object_layer.iter() {
                commands.entity(chunk_layer).despawn_recursive();
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
            init_tilemap(commands, asset_server, materials);

            next_state.set(GameState::Running);
        }
    }
}
