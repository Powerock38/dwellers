use std::{fs::File, io::Write, time::UNIX_EPOCH};

use bevy::{
    prelude::*,
    render::camera::{CameraMainTextureUsages, CameraRenderGraph},
    scene::SceneInstance,
    tasks::IoTaskPool,
};
use bevy_entitiles::{render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures};

use crate::{
    standard_tilemap_bundle, terrain::TilemapData, Dweller, Mob, Task, TileData, TERRAIN_SIZE,
};

pub const SAVE_DIR: &str = "saves";

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SaveName(pub String);

#[derive(Resource)]
pub struct SaveGame(pub String);

#[derive(Resource)]
pub struct LoadGame(pub String);

#[derive(Component)]
pub struct DynamicSceneForLoading;

pub fn save_key_shortcut(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    save_name: Res<SaveName>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        let timestamp = UNIX_EPOCH.elapsed().unwrap().as_millis();
        commands.insert_resource(SaveGame(format!("{}-{timestamp}", save_name.0.clone())));
    }
}

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

            // Save terrain with bitcode
            let tilemap_data_encoded = bitcode::encode(tilemap_data);

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
                    IoTaskPool::get()
                        .spawn(async move {
                            File::create(format!("assets/{SAVE_DIR}/{save_name}.ron"))
                                .and_then(|mut file| file.write(serialized.as_bytes()))
                                .expect("Error while writing entities to file");
                        })
                        .detach();

                    let save_name = save_game.0.clone();
                    IoTaskPool::get()
                        .spawn(async move {
                            File::create(format!("assets/{SAVE_DIR}/{save_name}.bin"))
                                .and_then(|mut file| file.write(tilemap_data_encoded.as_slice()))
                                .expect("Error while writing terrain to file");
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
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardTilemapMaterial>>,
    textures: ResMut<Assets<TilemapTextures>>,
    q_tilemap_data: Query<Entity, With<TilemapData>>,
    q_dwellers: Query<Entity, With<Dweller>>,
    q_tasks: Query<Entity, With<Task>>,
    q_mobs: Query<Entity, With<Mob>>,
) {
    if let Some(load_game) = load_game {
        if load_game.is_added() {
            commands.remove_resource::<LoadGame>();
            info!("Loading scene: {}", load_game.0);

            // Despawn current scene
            if let Some(tilemap_data) = q_tilemap_data.iter().next() {
                commands.entity(tilemap_data).despawn_recursive();
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
            commands.spawn((
                DynamicSceneForLoading,
                DynamicSceneBundle {
                    scene: asset_server.load(format!("{SAVE_DIR}/{}.ron", load_game.0.clone())),
                    ..default()
                },
            ));

            let entity = commands.spawn_empty().id();
            let mut tilemap = standard_tilemap_bundle(entity, asset_server, materials, textures);

            let tilemap_data = bitcode::decode::<TilemapData>(
                &std::fs::read(format!("assets/{SAVE_DIR}/{}.bin", load_game.0))
                    .expect("Error while reading terrain from file"),
            );

            if let Ok(tilemap_data) = tilemap_data {
                for x in 0..TERRAIN_SIZE {
                    for y in 0..TERRAIN_SIZE {
                        let index = IVec2::new(x as i32, y as i32);

                        if let Some(tile_data) = tilemap_data.get(index) {
                            tilemap
                                .storage
                                .set(&mut commands, index, tile_data.tile_builder());

                            TileData::update_light_level(
                                index,
                                &mut commands,
                                &mut tilemap.storage,
                                &tilemap_data,
                            );
                        }
                    }
                }
                commands.entity(entity).insert((tilemap, tilemap_data));
            } else {
                error!("Error while decoding terrain from file");
            }
        }
    }
}

// Called when DynamicSceneForLoading is fully loaded (== Added<SceneInstance>)
pub fn finish_load_world(
    mut commands: Commands,
    q_tilemap_data: Query<Entity, (Added<TilemapData>, With<Parent>)>,
    q_dynamic_scene: Query<Entity, (With<DynamicSceneForLoading>, Added<SceneInstance>)>,
) {
    let Some(tilemap_data) = q_tilemap_data.iter().next() else {
        return;
    };

    commands.entity(tilemap_data).remove_parent();

    if let Some(dynamic_scene) = q_dynamic_scene.iter().next() {
        commands.entity(dynamic_scene).despawn_recursive();
    }
}
