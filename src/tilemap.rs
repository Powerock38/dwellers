use bevy::{prelude::*, render::render_resource::FilterMode};
use bevy_entitiles::{
    prelude::*, render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures,
};
use bitcode::{Decode, Encode};

use crate::{TileData, TileKind, TERRAIN_SIZE};

pub const TILE_SIZE_U: u32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_U as f32;
const CHUNK_SIZE: u32 = 16;

#[derive(Clone, Copy)]
pub struct TilemapFile {
    name: &'static str,
    size: UVec2,
    start_atlas_index: i32,
}

#[derive(Component, Encode, Decode)]
pub struct TilemapData {
    data: Vec<Vec<TileData>>,
    size: usize,
}

impl TilemapData {
    pub fn new(fill_with: TileData, n: usize) -> Self {
        Self {
            data: vec![vec![fill_with; n]; n],
            size: n,
        }
    }

    pub fn get(&self, index: IVec2) -> Option<TileData> {
        if index.x < 0 || index.y < 0 {
            return None;
        }

        let x = index.x as usize;
        let y = index.y as usize;

        if x >= self.size || y >= self.size {
            return None;
        }

        Some(self.data[x][y])
    }

    pub fn set(&mut self, index: IVec2, value: TileData) {
        if index.x < 0 || index.y < 0 {
            return;
        }

        let x = index.x as usize;
        let y = index.y as usize;

        if x < self.size && y < self.size {
            self.data[x][y] = value;
        }
    }

    pub fn neighbours(&self, pos: IVec2) -> Vec<(IVec2, TileData)> {
        [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
            .into_iter()
            .filter_map(|p| {
                let index = pos + p;

                if let Some(tile) = self.get(index) {
                    return Some((index, tile));
                }

                None
            })
            .collect()
    }

    pub fn non_blocking_neighbours_pos(&self, pos: IVec2, diagonal: bool) -> Vec<IVec2> {
        let mut result: Vec<IVec2> = self
            .neighbours(pos)
            .into_iter()
            .filter_map(|(index, tile)| {
                if tile.is_blocking() {
                    None
                } else {
                    Some(index)
                }
            })
            .collect();

        if diagonal {
            let diagonal_directions = [
                IVec2::new(1, 1),
                IVec2::new(-1, 1),
                IVec2::new(1, -1),
                IVec2::new(-1, -1),
            ];

            for diag_pos in diagonal_directions {
                let diag_index = pos + diag_pos;

                if let Some(diag_tile) = self.get(diag_index) {
                    if !diag_tile.is_blocking() {
                        let adj_blocking = [IVec2::new(diag_pos.x, 0), IVec2::new(0, diag_pos.y)]
                            .into_iter()
                            .any(|adj| {
                                self.get(pos + adj)
                                    .map_or(true, |t| matches!(t.kind, TileKind::Wall))
                                // Do not allow diagonal movement if there is a wall, but allow if it's a blocking object
                            });

                        if !adj_blocking {
                            result.push(diag_index);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn find_from_center(index: IVec2, is_valid: impl Fn(IVec2) -> bool) -> Option<IVec2> {
        for radius in 0..=(TERRAIN_SIZE as i32 / 2) {
            // Top and Bottom edges of the square
            for x in (index.x - radius).max(1)..=(index.x + radius).min(TERRAIN_SIZE as i32 - 2) {
                // Top edge
                let top_y = (index.y - radius).max(1);
                if is_valid(IVec2::new(x, top_y)) {
                    return Some(IVec2::new(x, top_y));
                }
                // Bottom edge
                let bottom_y = (index.y + radius).min(TERRAIN_SIZE as i32 - 2);
                if is_valid(IVec2::new(x, bottom_y)) {
                    return Some(IVec2::new(x, bottom_y));
                }
            }

            // Left and Right edges of the square (excluding corners already checked)
            for y in ((index.y - radius + 1).max(1))
                ..=((index.y + radius - 1).min(TERRAIN_SIZE as i32 - 2))
            {
                // Left edge
                let left_x = (index.x - radius).max(1);
                if is_valid(IVec2::new(left_x, y)) {
                    return Some(IVec2::new(left_x, y));
                }
                // Right edge
                let right_x = (index.x + radius).min(TERRAIN_SIZE as i32 - 2);
                if is_valid(IVec2::new(right_x, y)) {
                    return Some(IVec2::new(right_x, y));
                }
            }
        }

        None
    }
}

pub fn standard_tilemap_bundle(
    entity: Entity,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) -> StandardTilemapBundle {
    StandardTilemapBundle {
        name: TilemapName("terrain".to_string()),
        tile_render_size: TileRenderSize(Vec2::splat(TILE_SIZE)),
        slot_size: TilemapSlotSize(Vec2::splat(TILE_SIZE)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(CHUNK_SIZE, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::new(
            TilemapFiles::T
                .files
                .iter()
                .map(|file| {
                    TilemapTexture::new(
                        asset_server.load(format!("tilemaps/{}.png", file.name)),
                        TilemapTextureDescriptor::new(
                            file.size * TILE_SIZE_U,
                            UVec2::splat(TILE_SIZE_U),
                        ),
                    )
                })
                .collect(),
            FilterMode::Nearest,
        )),
        ..default()
    }
}

const N: usize = 3;
pub type TF = (&'static str, UVec2);

pub struct TilemapFiles {
    files: [TilemapFile; N],
}

impl TilemapFiles {
    pub const T: Self = Self::new();

    pub const FLOORS: TF = ("floors", UVec2::new(2, 2));
    pub const WALLS: TF = ("walls", UVec2::new(4, 4));
    pub const OBJECTS: TF = ("objects", UVec2::new(4, 4));

    pub const fn new() -> Self {
        let tilemaps = [Self::FLOORS, Self::WALLS, Self::OBJECTS];

        let mut files = [TilemapFile {
            name: "",
            size: UVec2::ZERO,
            start_atlas_index: 0,
        }; N];

        let mut i = 0;
        let mut atlas_index = 0;

        while i < N {
            files[i] = TilemapFile {
                name: tilemaps[i].0,
                size: tilemaps[i].1,
                start_atlas_index: atlas_index,
            };
            atlas_index += (tilemaps[i].1.x * tilemaps[i].1.y) as i32;
            i += 1;
        }

        Self { files }
    }

    pub const fn atlas_index(&self, file: TF, atlas_index: i32) -> i32 {
        let mut i = 0;
        while i < N {
            // Can't compare strings directly, dirty workaround
            if self.files[i].name.len() == file.0.len() {
                return self.files[i].start_atlas_index + atlas_index;
            }
            i += 1;
        }
        0
    }
}
