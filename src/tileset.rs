use bevy::{asset::LoadedFolder, platform::collections::HashMap, prelude::*};

use crate::{Object, Tile, tilemap_chunk::TILE_SIZE_U};

#[derive(Resource)]
pub struct TilesFolder(pub Handle<LoadedFolder>);

pub fn init_tileset(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TilesFolder(asset_server.load_folder("tiles")));
}

pub fn wait_textures_load(
    mut commands: Commands,
    tiles_folder: Res<TilesFolder>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&tiles_folder.0) {
            let loaded_folder = loaded_folders.get(&tiles_folder.0).unwrap();
            let n_textures = loaded_folder.handles.len() as u32;

            // Build a texture atlas using the individual sprites
            // make it 2d array so we can call reinterpret_stacked_2d_as_array (necessary for TilemapChunk)
            let mut texture_atlas_builder = TextureAtlasBuilder::default();
            texture_atlas_builder.initial_size(UVec2::splat(TILE_SIZE_U));
            texture_atlas_builder.max_size(UVec2::new(TILE_SIZE_U, TILE_SIZE_U * n_textures));

            let mut paths_map = HashMap::new();

            for handle in &loaded_folder.handles {
                let id = handle.id().typed_unchecked::<Image>();
                let Some(texture) = textures.get(id) else {
                    warn!(
                        "{} did not resolve to an `Image` asset.",
                        handle.path().unwrap()
                    );
                    continue;
                };

                texture_atlas_builder.add_texture(Some(id), texture);

                let path = handle.path().unwrap().to_string();
                paths_map.insert(path, id);
            }

            let (texture_atlas_layout, texture_atlas_sources, mut texture) =
                texture_atlas_builder.build().unwrap();
            texture.reinterpret_stacked_2d_as_array(n_textures);
            let texture = textures.add(texture);
            texture_atlases.add(texture_atlas_layout);

            commands.insert_resource(Tileset {
                texture,
                texture_atlas_sources,
                paths_map,
            });
        }
    }
}

#[derive(Resource)]
pub struct Tileset {
    pub texture: Handle<Image>,
    texture_atlas_sources: TextureAtlasSources,
    paths_map: HashMap<String, AssetId<Image>>,
}

impl Tileset {
    pub fn get_atlas_index_tile(&mut self, tile: Tile) -> u16 {
        self.get_atlas_index(tile.sprite_path())
    }

    pub fn get_atlas_index_object(&mut self, object: Object) -> u16 {
        self.get_atlas_index(object.sprite_path())
    }

    fn get_atlas_index(&mut self, path: String) -> u16 {
        if let Some(id) = self.paths_map.get(&path)
            && let Some(index) = self.texture_atlas_sources.texture_index(*id)
        {
            return index as u16;
        }

        warn!(
            "Tile or Object with path `{}` not found in texture atlas.",
            path
        );
        0
    }
}
