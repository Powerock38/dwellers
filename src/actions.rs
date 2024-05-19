use bevy::{prelude::*, window::PrimaryWindow};
use bevy_entitiles::prelude::*;

use crate::{extract_ok, extract_some, terrain::TILE_SIZE};

//TODO: CurrentAction resource + Action enum

pub fn click_terrain(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut q_tilemap: Query<&mut TilemapStorage>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        let (camera, camera_transform) = extract_ok!(q_camera.get_single());
        let cursor_position = extract_some!(q_windows.single().cursor_position());
        let world_position =
            extract_some!(camera.viewport_to_world_2d(camera_transform, cursor_position));

        let mut tilemap = extract_ok!(q_tilemap.get_single_mut());

        let index = IVec2::new(
            (world_position.x / TILE_SIZE) as i32,
            (world_position.y / TILE_SIZE) as i32,
        );

        let tile = tilemap.get(index);

        // TODO: match CurrentAction
        if tile.is_some() {
            tilemap.set(
                &mut commands,
                index,
                TileBuilder::new()
                    .with_layer(0, TileLayer::no_flip(0))
                    .with_tint(Color::BLUE),
            );
        }
    }
}
