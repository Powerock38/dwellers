use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    SaveName, SpawnDwellersOnChunk, TilePlaced, TilemapData,
    dwellers::Dweller,
    generate_terrain, init_tilemap,
    mobs::Mob,
    random_text::{WORLD_NAMES, generate_word},
    sprites::SpriteLoader,
    tasks::{Task, TaskNeeds},
    tilemap::{CHUNK_SIZE, TILE_SIZE, Weather},
    utils::write_to_file,
};

#[derive(Message)]
pub struct LoadChunk(pub IVec2);

#[derive(Message)]
pub struct SaveChunk(pub IVec2, pub bool); // bool: despawn after save

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ChunksWithDwellers(pub HashSet<IVec2>);

pub fn spawn_new_terrain(
    mut commands: Commands,
    mut ev_load_chunk: MessageWriter<LoadChunk>,
    mut ev_spawn_dwellers: MessageWriter<SpawnDwellersOnChunk>,
) {
    let mut rng = rand::rng();
    let name = (0..2)
        .map(|_| {
            let mut word = generate_word(&WORLD_NAMES, &mut rng);
            word.get_mut(0..1).unwrap().make_ascii_uppercase();
            word
        })
        .collect::<String>();

    let save_name = SaveName(name);
    commands.insert_resource(Weather::new(save_name.seed()));
    commands.insert_resource(ChunksWithDwellers::default());
    init_tilemap(&mut commands, save_name);

    ev_load_chunk.write(LoadChunk(IVec2::ZERO));
    ev_spawn_dwellers.write(SpawnDwellersOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut ev_load: MessageReader<LoadChunk>,
    mut ev_save: MessageReader<SaveChunk>,
    save_name: Res<SaveName>,
    mut tilemap_data: ResMut<TilemapData>,
    q_save_entities: Query<(Entity, &Transform), Or<(With<Mob>, With<Task>, With<Dweller>)>>,
) {
    for LoadChunk(chunk_pos) in ev_load.read() {
        if tilemap_data.chunks.contains_key(chunk_pos) {
            continue;
        }

        let (path_bin, path_ron) = save_name.chunk_paths(*chunk_pos);
        let path_ron = path_ron.trim_start_matches("assets/").to_string();

        // Try to load the chunk from the save
        if let Some(chunk_data) = std::fs::read(path_bin)
            .ok()
            .and_then(|data| bitcode::decode::<Vec<TilePlaced>>(&data).ok())
        {
            debug!("Loading chunk {} from save file", chunk_pos);

            // Load in TilemapData
            tilemap_data.set_chunk(*chunk_pos, chunk_data);

            // Load entities in chunk with bevy reflection
            scene_spawner.spawn_dynamic(asset_server.load(path_ron));
        } else {
            // If the chunk is not in the save, generate it

            debug!("Generating chunk {}", chunk_pos);

            let chunk_data = generate_terrain(&mut commands, save_name.seed(), *chunk_pos);
            tilemap_data.set_chunk(*chunk_pos, chunk_data);
        }
    }

    for SaveChunk(chunk_pos, despawn) in ev_save.read() {
        let Some(chunk) = tilemap_data.chunks.get(chunk_pos) else {
            continue;
        };

        debug!("Saving chunk {}", chunk_pos);

        // Save chunk tiles with bitcode, save entities in chunk with bevy reflection
        let (path_bin, path_ron) = save_name.chunk_paths(*chunk_pos);

        let chunk_encoded = bitcode::encode(chunk);

        let chunk_min = chunk_pos.as_vec2() * CHUNK_SIZE as f32 * TILE_SIZE;
        let chunk_max = chunk_min + Vec2::splat(CHUNK_SIZE as f32 * TILE_SIZE);

        let entities = q_save_entities
            .iter()
            .filter(|(_, transform)| {
                let pos = transform.translation.truncate();
                pos.x >= chunk_min.x
                    && pos.x < chunk_max.x
                    && pos.y >= chunk_min.y
                    && pos.y < chunk_max.y
            })
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();

        // NOTE: we save Children components referencing entities that are not saved.
        // Could also happen with ChildOf, which may be even more problematic.

        let chunk_pos = *chunk_pos;
        let despawn = *despawn;
        commands.queue(move |world: &mut World| {
            let app_type_registry = world.resource::<AppTypeRegistry>().clone();

            let scene = DynamicSceneBuilder::from_world(world)
                .deny_all_resources()
                .deny_all_components()
                .allow_component::<Dweller>()
                .allow_component::<Mob>()
                .allow_component::<Task>() //FIXME: dwellers assigned to a task in another chunk are not correctly saved
                .allow_component::<TaskNeeds>()
                .allow_component::<SpriteLoader>()
                .allow_component::<Transform>()
                .allow_component::<GlobalTransform>()
                .allow_component::<Children>()
                .allow_component::<ChildOf>()
                .extract_entities(entities.into_iter())
                .remove_empty_entities()
                .build();

            let type_registry = app_type_registry.read();
            match scene.serialize(&type_registry) {
                Ok(serialized) => {
                    // blocking write to file to ensure data is saved before proceeding
                    write_to_file(path_bin, chunk_encoded);
                    write_to_file(path_ron, serialized.as_bytes());
                }
                Err(e) => {
                    error!("Error while serializing the scene: {e:?}");
                }
            }

            if despawn {
                debug!("Despawning chunk {}", chunk_pos);

                // Despawn entities in chunk
                for dyn_entity in scene.entities {
                    world.commands().entity(dyn_entity.entity).despawn();
                }

                // Remove chunk from TilemapData
                world.resource_mut::<TilemapData>().remove_chunk(chunk_pos);
            }
        });
    }
}
