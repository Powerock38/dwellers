use bevy::{prelude::*, sprite::Anchor};

use crate::TILE_SIZE;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpriteLoader {
    pub texture_path: String,
}

#[derive(Bundle, Default)]
pub struct SpriteLoaderBundle {
    pub loader: SpriteLoader,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl SpriteLoaderBundle {
    pub fn new(texture_path: impl Into<String>, x: f32, y: f32, z: f32) -> Self {
        Self {
            loader: SpriteLoader {
                texture_path: texture_path.into(),
            },
            transform: Transform::from_xyz(x, y, z),
            ..default()
        }
    }
}

pub fn scan_sprite_loaders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &SpriteLoader), Added<SpriteLoader>>,
) {
    for (entity, sprite_loader) in query.iter() {
        let handle: Handle<Image> = asset_server.load(sprite_loader.texture_path.clone());
        commands.entity(entity).insert((
            handle,
            Sprite {
                anchor: Anchor::BottomLeft,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
        ));
    }
}
