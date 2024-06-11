use bevy::{prelude::*, sprite::Anchor, window::PrimaryWindow};

use crate::{
    extract_ok, extract_some,
    mobs::Mob,
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    terrain::{TilemapData, TILE_SIZE},
    tiles::{ObjectData, TileKind},
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
            Self::Task(TaskKind::BuildObject { .. }) => write!(f, "Build Object"),
            Self::Task(task_kind) => write!(f, "{task_kind:?}"),
        }
    }
}

pub fn keyboard_current_action(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<CurrentAction>();
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
    q_mobs: Query<(Entity, &Transform), With<Mob>>,
) {
    if let Some(mut current_action) = current_action {
        if mouse_input.just_pressed(MouseButton::Right) {
            if current_action.index_start.is_some() {
                current_action.index_start = None;
            } else {
                commands.remove_resource::<CurrentAction>();
            }
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

                    // Make sure there is a tile at this position
                    let Some(tile_data) = tilemap_data.0.get(index) else {
                        continue;
                    };

                    match current_action.kind {
                        ActionKind::Cancel => {
                            if let Some((entity_task, task)) =
                                q_tasks.iter().find(|(_, task)| task.pos == index)
                            {
                                commands.entity(entity_task).despawn();

                                // if we are cancelling a stockpile task, mark item for pickup (if not already marked)
                                if task.kind == TaskKind::Stockpile
                                    && matches!(tile_data.kind, TileKind::Floor(Some(_)))
                                    && !q_tasks.iter().any(|(_, task)| {
                                        task.kind == TaskKind::Pickup && task.pos == index
                                    })
                                {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            TaskKind::Pickup,
                                            TaskNeeds::EmptyHands,
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/pickup.png"),
                                    ));

                                    println!("Removing stockpile at {index:?}");
                                } else {
                                    println!("Cancelling task at {index:?}");
                                }
                            }
                        }
                        ActionKind::Task(task_kind) => {
                            if !task_kind.can_be_completed(tile_data) {
                                continue;
                            }

                            // Abort if an incompatible task already exists at this position
                            if q_tasks.iter().filter(|(_, t)| t.pos == index).any(
                                |(entity_other_task, other_task)| match (task_kind, other_task.kind)
                                {
                                    (TaskKind::Stockpile, TaskKind::Pickup) => {
                                        commands.entity(entity_other_task).despawn();
                                        // Stockpile task will be correctly marked TaskNeeds::Impossible below
                                        false
                                    }
                                    _ => true,
                                },
                            ) {
                                continue;
                            }

                            match task_kind {
                                TaskKind::Dig => {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            task_kind,
                                            TaskNeeds::Nothing,
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/dig.png"),
                                    ));

                                    println!("Digging task at {index:?}");
                                }

                                TaskKind::Smoothen => {
                                    let task = Task::new(
                                        index,
                                        task_kind,
                                        TaskNeeds::Nothing,
                                        tilemap_data,
                                    );

                                    if !task.reachable_positions.is_empty() {
                                        commands.spawn(TaskBundle::new(
                                            task,
                                            asset_server.load("sprites/smoothen.png"),
                                        ));

                                        println!("Smoothening task at {index:?}");
                                    }
                                }

                                TaskKind::Chop => {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            task_kind,
                                            TaskNeeds::Nothing,
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/chop.png"),
                                    ));

                                    println!("Chopping task at {index:?}");
                                }

                                TaskKind::Bridge => {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            task_kind,
                                            TaskNeeds::Object(ObjectData::WOOD),
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/bridge.png"),
                                    ));

                                    println!("Building bridge task at {index:?}");
                                }

                                TaskKind::Pickup => {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            task_kind,
                                            TaskNeeds::EmptyHands,
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/pickup.png"),
                                    ));

                                    println!("Picking up task at {index:?}");
                                }

                                TaskKind::BuildObject { cost, .. } => {
                                    commands.spawn(TaskBundle::new(
                                        Task::new(
                                            index,
                                            task_kind,
                                            TaskNeeds::Object(cost),
                                            tilemap_data,
                                        ),
                                        asset_server.load("sprites/build.png"),
                                    ));

                                    println!("Building object task at {index:?}");
                                }

                                TaskKind::Hunt => {
                                    if let Some((entity_mob, _)) =
                                        q_mobs.iter().find(|(_, mob_transform)| {
                                            mob_transform.translation.distance(
                                                Vec3::new(index.x as f32, index.y as f32, 0.)
                                                    * TILE_SIZE,
                                            ) < TILE_SIZE
                                        })
                                    {
                                        commands.entity(entity_mob).with_children(|c| {
                                            c.spawn(TaskBundle {
                                                task: Task::new(
                                                    index,
                                                    task_kind,
                                                    TaskNeeds::Nothing,
                                                    tilemap_data,
                                                ),
                                                sprite: SpriteBundle {
                                                    texture: asset_server.load("sprites/hunt.png"),
                                                    sprite: Sprite {
                                                        anchor: Anchor::BottomLeft,
                                                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                                                        ..default()
                                                    },
                                                    transform: Transform::from_xyz(0., 0., 1.),
                                                    ..default()
                                                },
                                            });
                                        });

                                        println!("Hunting task at {index:?}");
                                    }
                                }

                                TaskKind::Stockpile => {
                                    let mut task = Task::new(
                                        index,
                                        task_kind,
                                        if tile_data.kind == TileKind::Floor(None) {
                                            TaskNeeds::AnyObject
                                        } else {
                                            TaskNeeds::Impossible
                                        },
                                        tilemap_data,
                                    );

                                    task.priority(-1);

                                    commands.spawn(TaskBundle::new(
                                        task,
                                        asset_server.load("sprites/stockpile.png"),
                                    ));

                                    println!("Stockpiling task at {index:?}");
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
