use bevy::prelude::*;

use crate::{
    GameState, init_tilemap,
    tilemap::{ChunksWithDwellers, LoadChunk, Weather},
    utils::write_to_file,
};

pub const SAVE_DIR: &str = "saves";

#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct SaveName(pub String);

impl SaveName {
    pub fn seed(&self) -> u32 {
        self.0.as_bytes().iter().map(|b| *b as u32).sum()
    }

    fn folder(&self) -> String {
        format!("assets/{}/{}", SAVE_DIR, self.0)
    }

    pub fn chunk_paths(&self, chunk_pos: IVec2) -> (String, String) {
        let base = format!("{}/{}_{}", self.folder(), chunk_pos.x, chunk_pos.y);
        (format!("{base}.bin"), format!("{base}.ron"))
    }

    pub fn resources_path(&self) -> String {
        format!("{}/resources.ron", self.folder())
    }
}

#[derive(Event)]
pub struct SaveResources;

#[derive(Event)]
pub struct LoadGame(pub String);

#[derive(Component, Default)]
pub struct SaveScoped;

pub fn save_resources(
    _: On<SaveResources>,
    mut commands: Commands,
    save_name: Res<SaveName>,
    world: &World,
) {
    // Save resources with bevy reflection
    debug!("Saving resources: {}", save_name.0);

    let app_type_registry = world.resource::<AppTypeRegistry>().clone();

    // Small caveat: we save Children components referencing entities that are not saved.
    // Could also happen with ChildOf, which may be even more problematic.

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all_resources()
        .allow_resource::<Weather>()
        .allow_resource::<ChunksWithDwellers>()
        .extract_resources()
        .build();

    let type_registry = app_type_registry.read();
    match scene.serialize(&type_registry) {
        Ok(serialized) => {
            // Save tasks & entities with Bevy reflection
            write_to_file(save_name.resources_path(), serialized.as_bytes());
        }
        Err(e) => {
            error!("Error while serializing the scene: {e:?}");
        }
    }

    // Can't have mut next_state: ResMut<NextState<GameState>> in this system because of world borrow
    commands.queue(|world: &mut World| {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Running);
    });
}

pub fn load_game(
    load_game: On<LoadGame>,
    mut commands: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
    q_save_scoped: Query<Entity, With<SaveScoped>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("Loading game: {}", load_game.0);

    // Despawn current scene
    for save_scoped in q_save_scoped.iter() {
        commands.entity(save_scoped).despawn();
    }
    commands.remove_resource::<ChunksWithDwellers>();
    commands.remove_resource::<Weather>();

    let save_name = SaveName(load_game.0.clone());

    // Spawn resources from ron file
    let resources_path = save_name
        .resources_path()
        .trim_start_matches("assets/")
        .to_string();
    scene_spawner.spawn_dynamic(asset_server.load(resources_path));

    // Init tilemap
    init_tilemap(&mut commands, save_name);

    next_state.set(GameState::Running);
}

pub fn chunks_with_dwellers_is_added(
    mut commands: Commands,
    chunks_with_dwellers: If<Res<ChunksWithDwellers>>,
) {
    // This is triggered when loading a save
    if chunks_with_dwellers.is_added() {
        debug!("Loading ChunksWithDwellers {:?}", (*chunks_with_dwellers).0);
        for chunk_pos in &(*chunks_with_dwellers).0 {
            commands.write_message(LoadChunk(*chunk_pos));
        }
    }
}
