use bevy::{prelude::*, window::PrimaryWindow};
use bevy_entitiles::prelude::*;

use crate::{
    extract_ok, extract_some,
    tasks::Task,
    terrain::{TilemapData, TILE_SIZE},
};

#[derive(Resource)]
pub enum CurrentAction {
    Dig,
}

pub fn keyboard_current_action(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<CurrentAction>();
    } else if keyboard_input.just_pressed(KeyCode::KeyZ) {
        commands.insert_resource(CurrentAction::Dig);
    }
}

pub fn click_terrain(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    current_action: Option<Res<CurrentAction>>,
    mut q_tilemap: Query<(&mut TilemapStorage, &TilemapData)>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        let (camera, camera_transform) = extract_ok!(q_camera.get_single());
        let cursor_position = extract_some!(q_windows.single().cursor_position());
        let world_position =
            extract_some!(camera.viewport_to_world_2d(camera_transform, cursor_position));

        let (mut tilemap, tilemap_data) = extract_ok!(q_tilemap.get_single_mut());

        let index = IVec2::new(
            (world_position.x / TILE_SIZE) as i32,
            (world_position.y / TILE_SIZE) as i32,
        );

        let tile = tilemap.get(index);

        let tile_data = extract_some!(tilemap_data.0.get(index));

        if let Some(current_action) = current_action {
            match *current_action {
                CurrentAction::Dig => {
                    if tile_data.wall {
                        commands.spawn(Task::Dig(index));

                        /* if tile.is_some() {
                            tilemap.set(
                                &mut commands,
                                index,
                                TileBuilder::new()
                                    .with_layer(0, TileLayer::no_flip(0))
                                    .with_tint(Color::BLUE),
                            );
                        } */
                    }
                }
            }
        }
    }
}
