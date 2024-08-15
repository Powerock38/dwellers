use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    data::ObjectId,
    extract_ok, extract_some,
    mobs::Mob,
    tasks::{Task, TaskBundle, TaskKind, TaskNeeds},
    tilemap::{TilemapData, TILE_SIZE},
    Dweller, DwellersSelected,
};

const MAX_ACTIONS: usize = 256;

#[derive(PartialEq, Clone, Debug)]
pub enum ActionKind {
    Select,
    Cancel,
    Task(TaskKind),
    TaskWithNeeds(TaskKind, TaskNeeds),
}

impl std::fmt::Display for ActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Select => write!(f, ""),
            Self::Cancel => write!(f, "{self:?}"),
            Self::Task(task_kind) | Self::TaskWithNeeds(task_kind, _) => {
                write!(
                    f,
                    "{}",
                    format!("{task_kind:?}").split_whitespace().next().unwrap()
                )
            }
        }
    }
}

pub fn keyboard_current_action(
    mut commands: Commands,
    mut current_action: ResMut<CurrentAction>,
    mut dwellers_selected: ResMut<DwellersSelected>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if current_action.index_start.is_some() {
            current_action.index_start = None;
        } else {
            if matches!(current_action.kind, ActionKind::Select) {
                dwellers_selected.reset();
            }
            commands.insert_resource(CurrentAction::default());
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

impl Default for CurrentAction {
    fn default() -> Self {
        Self::new(ActionKind::Select)
    }
}

pub fn click_terrain(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut current_action: ResMut<CurrentAction>,
    mut dwellers_selected: ResMut<DwellersSelected>,
    q_tilemap: Query<&TilemapData>,
    q_tasks: Query<(Entity, &Task)>,
    q_mobs: Query<(Entity, &Transform), With<Mob>>,
    q_dwellers: Query<(Entity, &Transform), With<Dweller>>,
) {
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

        let mut max_tasks = match current_action.kind {
            ActionKind::Task(TaskKind::Walk) => {
                if dwellers_selected.list().is_empty() {
                    q_dwellers.iter().len()
                } else {
                    dwellers_selected.list().len()
                }
            }
            _ => MAX_ACTIONS,
        };

        if matches!(current_action.kind, ActionKind::Select) {
            dwellers_selected.reset();
        }

        'index: for y in (index_min.y..=index_max.y).rev() {
            for x in index_min.x..=index_max.x {
                let index = IVec2::new(x, y);

                if max_tasks == 0 {
                    break 'index;
                }

                // Make sure there is a tile at this position
                let Some(tile) = tilemap_data.get(index) else {
                    continue;
                };

                // If Task, check validity
                if let ActionKind::Task(task_kind) | ActionKind::TaskWithNeeds(task_kind, _) =
                    current_action.kind
                {
                    if !task_kind.is_valid_on_tile(tile) {
                        continue;
                    }

                    // Abort if an incompatible task already exists at this position
                    if q_tasks.iter().filter(|(_, t)| t.pos == index).any(
                        |(entity_other_task, other_task)| match (task_kind, other_task.kind) {
                            (TaskKind::Stockpile, TaskKind::Pickup) => {
                                commands.entity(entity_other_task).despawn();
                                // Stockpile task will be correctly marked TaskNeeds::Impossible below
                                false
                            }
                            (TaskKind::Pickup, TaskKind::Stockpile)
                            | (
                                TaskKind::Smoothen,
                                TaskKind::Stockpile
                                | TaskKind::Pickup
                                | TaskKind::Harvest
                                | TaskKind::Hunt
                                | TaskKind::Workstation { .. },
                            ) => false,
                            _ => true,
                        },
                    ) {
                        continue;
                    }
                }

                let dweller = dwellers_selected.next();

                match &current_action.kind {
                    ActionKind::Task(task_kind) => match task_kind {
                        TaskKind::Dig => {
                            commands.spawn(TaskBundle::new(Task::new(
                                index,
                                *task_kind,
                                TaskNeeds::Nothing,
                                dweller,
                                tilemap_data,
                            )));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Digging task at {index:?}");
                        }

                        TaskKind::Smoothen => {
                            let task = Task::new(
                                index,
                                *task_kind,
                                TaskNeeds::Nothing,
                                dweller,
                                tilemap_data,
                            );

                            if !task.reachable_positions.is_empty() {
                                commands.spawn(TaskBundle::new(task));

                                max_tasks = max_tasks.saturating_sub(1);
                                debug!("Smoothening task at {index:?}");
                            }
                        }

                        TaskKind::Harvest => {
                            commands.spawn(TaskBundle::new(Task::new(
                                index,
                                *task_kind,
                                match tile.object {
                                    Some(ObjectId::WheatPlant) => TaskNeeds::EmptyHands,
                                    _ => TaskNeeds::Nothing,
                                },
                                dweller,
                                tilemap_data,
                            )));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Harvesting task at {index:?}");
                        }

                        TaskKind::Bridge => {
                            commands.spawn(TaskBundle::new(Task::new(
                                index,
                                *task_kind,
                                TaskNeeds::Objects(vec![ObjectId::Wood]),
                                dweller,
                                tilemap_data,
                            )));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Building bridge task at {index:?}");
                        }

                        TaskKind::Pickup => {
                            commands.spawn(TaskBundle::new(Task::new(
                                index,
                                *task_kind,
                                TaskNeeds::EmptyHands,
                                dweller,
                                tilemap_data,
                            )));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Picking up task at {index:?}");
                        }

                        TaskKind::Hunt => {
                            if let Some((entity_mob, _)) =
                                q_mobs.iter().find(|(_, mob_transform)| {
                                    mob_transform.translation.distance(
                                        Vec3::new(index.x as f32, index.y as f32, 0.) * TILE_SIZE,
                                    ) < TILE_SIZE
                                })
                            {
                                commands.entity(entity_mob).with_children(|c| {
                                    c.spawn(TaskBundle::new_as_child(Task::new(
                                        index,
                                        *task_kind,
                                        TaskNeeds::Nothing,
                                        dweller,
                                        tilemap_data,
                                    )));
                                });

                                max_tasks = max_tasks.saturating_sub(1);
                                debug!("Hunting task at {index:?}");
                            }
                        }

                        TaskKind::Stockpile => {
                            let mut task = Task::new(
                                index,
                                *task_kind,
                                if tile.object.is_none() {
                                    TaskNeeds::AnyObject
                                } else {
                                    TaskNeeds::Impossible
                                },
                                dweller,
                                tilemap_data,
                            );

                            task.priority(-1);

                            commands.spawn(TaskBundle::new(task));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Stockpiling task at {index:?}");
                        }

                        TaskKind::Walk => {
                            let mut task = Task::new(
                                index,
                                *task_kind,
                                TaskNeeds::Nothing,
                                dweller,
                                tilemap_data,
                            );

                            task.priority(-1);

                            commands.spawn(TaskBundle::new(task));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Going there task at {index:?}");
                        }

                        TaskKind::Workstation { .. } | TaskKind::Build { .. } => {}
                    },

                    ActionKind::TaskWithNeeds(task_kind, needs) => match task_kind {
                        TaskKind::Build { .. } => {
                            commands.spawn(TaskBundle::new(Task::new(
                                index,
                                *task_kind,
                                needs.clone(),
                                dweller,
                                tilemap_data,
                            )));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Building task at {index:?}");
                        }

                        _ => {}
                    },

                    ActionKind::Cancel => {
                        if let Some((entity_task, task)) =
                            q_tasks.iter().find(|(_, task)| task.pos == index)
                        {
                            commands.entity(entity_task).despawn();

                            // if we are cancelling a stockpile task, mark item for pickup (if not already marked)
                            if task.kind == TaskKind::Stockpile
                                && TaskKind::Pickup.is_valid_on_tile(tile)
                                && !q_tasks.iter().any(|(_, task)| {
                                    task.kind == TaskKind::Pickup && task.pos == index
                                })
                            {
                                commands.spawn(TaskBundle::new(Task::new(
                                    index,
                                    TaskKind::Pickup,
                                    TaskNeeds::EmptyHands,
                                    dweller,
                                    tilemap_data,
                                )));

                                debug!("Removing stockpile at {index:?}");
                            } else {
                                debug!("Cancelling task at {index:?}");
                            }
                        }
                    }

                    ActionKind::Select => {
                        for (entity, transform) in &q_dwellers {
                            if index.x == (transform.translation.x / TILE_SIZE) as i32
                                && index.y == (transform.translation.y / TILE_SIZE) as i32
                            {
                                dwellers_selected.add(entity);
                            }
                        }
                    }
                }
            }
        }

        current_action.index_start = None;
    }
}
