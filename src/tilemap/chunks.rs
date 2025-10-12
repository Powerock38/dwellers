use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{
    SAVE_DIR, SaveName, SpawnDwellersOnChunk, TilePlaced, TilemapData, generate_terrain,
    init_tilemap,
    random_text::{WORLD_NAMES, generate_word},
    utils::write_to_file,
};

#[derive(Message)]
pub struct LoadChunk(pub IVec2);

#[derive(Message)]
pub struct UnloadChunk(pub IVec2);

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

    commands.insert_resource(SaveName(name));
    init_tilemap(commands);

    ev_load_chunk.write(LoadChunk(IVec2::ZERO));
    ev_spawn_dwellers.write(SpawnDwellersOnChunk(IVec2::ZERO));
}

pub fn load_chunks(
    mut commands: Commands,
    mut ev_load: MessageReader<LoadChunk>,
    mut ev_unload: MessageReader<UnloadChunk>,
    mut tilemap_data: ResMut<TilemapData>,
    save_name: Res<SaveName>,
) {
    // Seed is based on the save name
    let seed = save_name.seed();

    let save_folder = format!("assets/{SAVE_DIR}/{}", save_name.0);

    let mut loaded = vec![];

    for LoadChunk(chunk_pos) in ev_load.read() {
        if loaded.contains(chunk_pos) {
            continue;
        }

        loaded.push(*chunk_pos);

        if tilemap_data.chunks.contains_key(chunk_pos) {
            continue;
        }

        // Try to load the chunk from the save
        if let Some(chunk_data) =
            std::fs::read(format!("{save_folder}/{}_{}.bin", chunk_pos.x, chunk_pos.y))
                .ok()
                .and_then(|data| bitcode::decode::<Vec<TilePlaced>>(&data).ok())
        {
            debug!("Loading chunk {} from save file", chunk_pos);

            // Load in TilemapData
            tilemap_data.set_chunk(*chunk_pos, chunk_data);
        } else {
            // If the chunk is not in the save, generate it

            debug!("Generating chunk {}", chunk_pos);

            let chunk_data = generate_terrain(&mut commands, seed, *chunk_pos);
            tilemap_data.set_chunk(*chunk_pos, chunk_data);
        }
    }

    for UnloadChunk(chunk_pos) in ev_unload.read() {
        let Some(chunk) = tilemap_data.chunks.get(chunk_pos) else {
            continue;
        };

        debug!("Unloading chunk {}", chunk_pos);

        let chunk_encoded = bitcode::encode(chunk);

        let save_folder = save_folder.clone();
        let x = chunk_pos.x;
        let y = chunk_pos.y;

        IoTaskPool::get()
            .spawn(async move {
                let path = format!("{save_folder}/{x}_{y}.bin");
                write_to_file(path, chunk_encoded);
            })
            .detach();
        tilemap_data.remove_chunk(*chunk_pos);
    }
}
