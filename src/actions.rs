use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    extract_ok, extract_some,
    tasks::{Task, TaskBundle, TaskKind},
    terrain::{TilemapData, TILE_SIZE},
    tiles::TileData,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActionKind {
    Cancel,
    Task(TaskKind),
}

impl std::fmt::Display for ActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "{self:?}"),
            Self::Task(task_kind) => write!(f, "{task_kind:?}"),
        }
    }
}

#[derive(Resource, Debug)]
pub struct CurrentAction {
    pub kind: ActionKind,
    pub index_start: Option<IVec2>,
}

impl CurrentAction {
    pub fn new(kind: ActionKind) -> Self {
        Self {
            kind,
            index_start: None,
        }
    }
}

pub fn keyboard_current_action(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<CurrentAction>();
    } else if keyboard_input.just_pressed(KeyCode::KeyX) {
        commands.insert_resource(CurrentAction::new(ActionKind::Task(TaskKind::Dig)));
    } else if keyboard_input.just_pressed(KeyCode::KeyZ) {
        commands.insert_resource(CurrentAction::new(ActionKind::Task(TaskKind::Smoothen)));
    } else if keyboard_input.just_pressed(KeyCode::KeyC) {
        commands.insert_resource(CurrentAction::new(ActionKind::Cancel));
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
    q_tasks: Query<(Entity, &Task)>,
) {
    if let Some(mut current_action) = current_action {
        if mouse_input.just_pressed(MouseButton::Right) {
            current_action.index_start = None;
            return;
        }

        let (camera, camera_transform) = extract_ok!(q_camera.get_single());
        let window = extract_ok!(q_windows.get_single());
        let cursor_position = extract_some!(window.cursor_position());
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
            let index_start = extract_some!(current_action.index_start);

            let index_min = IVec2::new(index_start.x.min(index.x), index_start.y.min(index.y));
            let index_max = IVec2::new(index_start.x.max(index.x), index_start.y.max(index.y));

            for x in index_min.x..=index_max.x {
                for y in index_min.y..=index_max.y {
                    let index = IVec2::new(x, y);

                    match current_action.kind {
                        ActionKind::Cancel => {
                            let entity_task = q_tasks.iter().find_map(|(entity, task)| {
                                if task.pos == index {
                                    Some(entity)
                                } else {
                                    None
                                }
                            });

                            if let Some(entity_task) = entity_task {
                                commands.entity(entity_task).despawn();
                                println!("Cancelling task at {index:?}");
                            }
                        }
                        ActionKind::Task(task_kind) => {
                            // Make sure there is a tile at this position
                            let Some(tile_data) = tilemap_data.0.get(index) else {
                                continue;
                            };

                            // Abort if a task already exists at this position
                            if q_tasks.iter().any(|(_, task)| task.pos == index) {
                                continue;
                            }

                            match task_kind {
                                TaskKind::Dig => {
                                    if tile_data == TileData::DIRT_WALL
                                        || tile_data == TileData::STONE_WALL
                                    {
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
                                        let task =
                                            Task::new(index, TaskKind::Smoothen, tilemap_data);

                                        if !task.reachable_positions.is_empty() {
                                            commands.spawn(TaskBundle::new(
                                                task,
                                                asset_server.load("sprites/smoothen.png"),
                                            ));

                                            println!("Smoothening task at {index:?}");
                                        }
                                    }
                                }

                                TaskKind::Chop => {
                                    if tile_data == TileData::TREE {
                                        commands.spawn(TaskBundle::new(
                                            Task::new(index, TaskKind::Chop, tilemap_data),
                                            asset_server.load("sprites/chop.png"),
                                        ));

                                        println!("Chopping task at {index:?}");
                                    }
                                }

                                TaskKind::Bridge => {
                                    if tile_data == TileData::WATER {
                                        commands.spawn(TaskBundle::new(
                                            Task::new(index, TaskKind::Bridge, tilemap_data),
                                            asset_server.load("sprites/bridge.png"),
                                        ));

                                        println!("Building bridge task at {index:?}");
                                    }
                                }
                            }
                        }
                    }
                }
            }

            current_action.index_start = None;
        }
    }
}
