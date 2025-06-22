use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{
    init_tilemap, save_load::SpriteLoader, tasks::TaskNeeds, tilemap_data::TilemapData,
    utils::write_to_file, ChunkObjectLayer, ChunkTileLayer, Dweller, GameState, Mob, Task,
};

pub const SAVE_DIR: &str = "saves";

#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct SaveName(pub String);

#[derive(Resource)]
pub struct SaveGame;

#[derive(Resource)]
pub struct LoadGame(pub String);

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

            // Small caveat: we save Children components referencing entities that are not saved.
            // Could also happen with ChildOf, which may be even more problematic.

            let scene = DynamicSceneBuilder::from_world(world)
                .deny_all_resources()
                .deny_all_components()
                .allow_resource::<SaveName>()
                .allow_component::<Dweller>()
                .allow_component::<Mob>()
                .allow_component::<Task>()
                .allow_component::<TaskNeeds>()
                .allow_component::<SpriteLoader>()
                .allow_component::<Transform>()
                .allow_component::<GlobalTransform>()
                .allow_component::<Children>()
                .allow_component::<ChildOf>()
                .extract_resources()
                .extract_entities(q_dwellers.iter())
                .extract_entities(q_tasks.iter())
                .extract_entities(q_mobs.iter())
                .remove_empty_entities()
                .build();

            let type_registry = app_type_registry.read();
            match scene.serialize(&type_registry) {
                Ok(serialized) => {
                    let path = format!("assets/{SAVE_DIR}/{}/entities.ron", save_name.0);

                    // Save tasks & entities with Bevy reflection
                    IoTaskPool::get()
                        .spawn(async move {
                            write_to_file(path, serialized.as_bytes());
                        })
                        .detach();
                }
                Err(e) => {
                    error!("Error while serializing the scene: {e:?}");
                }
            }

            commands.queue(|world: &mut World| {
                let mut next_state = world.resource_mut::<NextState<GameState>>();
                next_state.set(GameState::Running);
            });
        }
    }
}

pub fn load_world(
    mut commands: Commands,
    load_game: Option<Res<LoadGame>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
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
                commands.entity(chunk_layer).despawn();
            }

            for chunk_layer in q_chunks_object_layer.iter() {
                commands.entity(chunk_layer).despawn();
            }

            for dweller in q_dwellers.iter() {
                commands.entity(dweller).despawn();
            }

            for task in q_tasks.iter() {
                commands.entity(task).despawn();
            }

            for mob in q_mobs.iter() {
                commands.entity(mob).despawn();
            }

            // Spawn new scene
            scene_spawner.spawn_dynamic(
                asset_server.load(format!("{SAVE_DIR}/{}/entities.ron", load_game.0.clone())),
            );

            // Init tilemap, chunks will be loaded from disk
            init_tilemap(commands, asset_server);

            next_state.set(GameState::Running);
        }
    }
}
