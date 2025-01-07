use bevy::{prelude::*, sprite::Anchor};

use crate::TILE_SIZE;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct SpriteLoader {
    pub texture_path: String,
}

pub fn scan_sprite_loaders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &SpriteLoader), Added<SpriteLoader>>,
) {
    for (entity, sprite_loader) in query.iter() {
        let image = asset_server.load(sprite_loader.texture_path.clone());
        commands.entity(entity).insert(Sprite {
            image,
            anchor: Anchor::BottomLeft,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        });
    }
}
