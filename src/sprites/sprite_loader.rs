use bevy::{prelude::*, sprite::Anchor};

use crate::{TILE_SIZE, TILE_SIZE_U, data::SPRITE_ANIMATIONS};

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Transform, Visibility)]
pub struct SpriteLoader {
    pub texture_path: String,
}

#[derive(Component)]
pub struct SpriteAnimation {
    pub timer: Timer,
    pub last_frame: usize,
}

pub fn scan_sprites_loaders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &SpriteLoader), Added<SpriteLoader>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, sprite_loader) in query.iter() {
        let image = asset_server.load(&sprite_loader.texture_path);

        let (n_frames, duration) = SPRITE_ANIMATIONS
            .get(sprite_loader.texture_path.as_str())
            .copied()
            .unwrap_or((1, 0.2));

        if n_frames > 1 {
            let layout =
                TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE_U), n_frames, 1, None, None);

            commands.entity(entity).insert((
                Sprite::from_atlas_image(
                    image,
                    TextureAtlas {
                        layout: texture_atlas_layouts.add(layout),
                        index: 0,
                    },
                ),
                SpriteAnimation {
                    timer: Timer::from_seconds(duration, TimerMode::Repeating),
                    last_frame: n_frames as usize - 1,
                },
            ));
        } else {
            commands.entity(entity).insert(Sprite {
                image,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            });
        }

        commands.entity(entity).insert(Anchor::BOTTOM_LEFT);
    }
}

pub fn update_sprite_animation(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == anim.last_frame {
                0
            } else {
                atlas.index + 1
            };
        }
    }
}
