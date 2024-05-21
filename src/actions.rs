use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    extract_ok, extract_some,
    tasks::{Task, TaskBundle, TaskKind},
    terrain::{TilemapData, TILE_SIZE},
    tiles::TileData,
};

#[derive(Resource, Debug)]
pub struct CurrentAction {
    pub task_kind: TaskKind,
    pub index_start: Option<IVec2>,
}

impl CurrentAction {
    pub fn new(task_kind: TaskKind) -> Self {
        Self {
            task_kind,
            index_start: None,
        }
    }
}

pub fn keyboard_current_action(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<CurrentAction>();
    } else if keyboard_input.just_pressed(KeyCode::KeyX) {
        commands.insert_resource(CurrentAction::new(TaskKind::Dig));
    } else if keyboard_input.just_pressed(KeyCode::KeyZ) {
        commands.insert_resource(CurrentAction::new(TaskKind::Smoothen));
    }
}

pub fn click_terrain(
    mut commands: Commands,
    mut gizmos: Gizmos,
    asset_server: Res<AssetServer>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    current_action: Option<ResMut<CurrentAction>>,
    q_tilemap: Query<&TilemapData>,
    q_tasks: Query<&Task>,
) {
    if let Some(mut current_action) = current_action {
        let (camera, camera_transform) = extract_ok!(q_camera.get_single());
        let cursor_position = extract_some!(q_windows.single().cursor_position());
        let world_position =
            extract_some!(camera.viewport_to_world_2d(camera_transform, cursor_position));

        let tilemap_data = extract_ok!(q_tilemap.get_single());

        let index = IVec2::new(
            (world_position.x / TILE_SIZE) as i32,
            (world_position.y / TILE_SIZE) as i32,
        );

        if let Some(index_start) = current_action.index_start {
            let from = Vec2::new(index_start.x as f32, index_start.y as f32) * TILE_SIZE;
            let to = Vec2::new(index.x as f32, index.y as f32) * TILE_SIZE;

            let center = (from + to) / 2. + TILE_SIZE / 2.;
            let size = (to - from).abs() + TILE_SIZE / 2.;

            gizmos.rect_2d(center, 0., size, Color::WHITE);
        }

        if mouse_input.just_pressed(MouseButton::Left) {
            // Start selection
            current_action.index_start = Some(index);
        }

        if mouse_input.just_released(MouseButton::Left) {
            // End selection
            if let Some(index_start) = current_action.index_start {
                let index_min = IVec2::new(index_start.x.min(index.x), index_start.y.min(index.y));
                let index_max = IVec2::new(index_start.x.max(index.x), index_start.y.max(index.y));

                for x in index_min.x..=index_max.x {
                    for y in index_min.y..=index_max.y {
                        let index = IVec2::new(x, y);
                        let tile_data = extract_some!(tilemap_data.0.get(index));

                        // Check if task already exists at this position
                        if q_tasks.iter().any(|task| task.pos == index) {
                            continue;
                        }

                        match current_action.task_kind {
                            TaskKind::Dig => {
                                if tile_data.is_blocking() {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(index, TaskKind::Dig, tilemap_data),
                                        asset_server.load("sprites/dig.png"),
                                    ));

                                    println!("Digging task at {index:?}");
                                }
                            }
                            TaskKind::Smoothen => {
                                if tile_data == TileData::DIRT_WALL
                                    || tile_data == TileData::STONE_WALL
                                    || tile_data == TileData::STONE_FLOOR
                                {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(index, TaskKind::Smoothen, tilemap_data),
                                        asset_server.load("sprites/smoothen.png"),
                                    ));

                                    println!("Smoothening task at {index:?}");
                                }
                            }
                        }
                    }
                }

                current_action.index_start = None;
            }
        }
    }
}
