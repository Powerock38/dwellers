use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{
    init_tilemap,
    random_text::{generate_word, WORLD_NAMES},
    terrain::generate_terrain,
    tilemap_data::TilemapData,
    tiles::TilePlaced,
    utils::write_to_file,
    SaveName, SpawnDwellersOnChunk, SAVE_DIR,
};

#[derive(Event)]
pub struct LoadChunk(pub IVec2);

#[derive(Event)]
pub struct UnloadChunk(pub IVec2);

pub fn spawn_new_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_load_chunk: EventWriter<LoadChunk>,
    mut ev_spawn_dwellers: EventWriter<SpawnDwellersOnChunk>,
) {
    let mut rng = rand::rng();
    let name = (0..2)
        .map(|_| {
            let mut word = generate_word(&WORLD_NAMES, &mut rng);
            word.get_mut(0..1).unwrap().make_ascii_uppercase();
            word
        })
        .collect::<String>();

    commands.insert_resource(SaveName(name));
    init_tilemap(commands, asset_server);

    ev_load_chunk.write(LoadChunk(IVec2::ZERO));
    ev_spawn_dwellers.write(SpawnDwellersOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut ev_load: EventReader<LoadChunk>,
    mut ev_unload: EventReader<UnloadChunk>,
    mut tilemap_data: ResMut<TilemapData>,
    save_name: Res<SaveName>,
) {
    // Seed is based on the save name
    let seed = save_name.0.as_bytes().iter().map(|b| *b as u32).sum();

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

            let chunk_data = generate_terrain(&mut commands, seed, *chunk_index);
            tilemap_data.set_chunk(*chunk_index, chunk_data);
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
