use bevy::prelude::*;

use crate::{
    data::{MobId, ObjectId},
    extract_ok, extract_some,
    mobs::{Mob, MobBundle},
    tasks::{BuildResult, Task, TaskBundle, TaskKind, TaskNeeds},
    tilemap::TILE_SIZE,
    tilemap_data::TilemapData,
    ui::{CoordinatesUi, UiButton},
    utils::transform_to_index,
    Dweller, DwellersSelected, OpenWorkstationUi,
};

const MAX_ACTIONS: usize = 2048;

#[derive(PartialEq, Clone, Default, Debug)]
pub enum ActionKind {
    #[default]
    Select,
    Cancel,
    Task(TaskKind),
    TaskWithNeeds(TaskKind, TaskNeeds),
    DebugBuild(BuildResult),
    DebugSpawn(MobId),
}

pub fn keyboard_current_action(
    mut commands: Commands,
    mut current_action: ResMut<CurrentAction>,
    mut dwellers_selected: ResMut<DwellersSelected>,
    mut q_borders: Query<&mut BorderColor, With<UiButton>>,
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

            for mut border in &mut q_borders {
                border.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Resource, Default, Debug)]
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

pub fn terrain_pointer_down(
    trigger: Trigger<Pointer<Pressed>>,
    mut current_action: ResMut<CurrentAction>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_ui_buttons: Query<(), With<UiButton>>,
    mut coordinates_ui: Single<&mut Text, With<CoordinatesUi>>,
) {
    if q_ui_buttons.contains(trigger.target()) {
        return;
    }

    match trigger.button {
        PointerButton::Primary => {
            let (camera, camera_transform) = extract_ok!(q_camera.single());
            let world_position =
                extract_ok!(camera
                    .viewport_to_world_2d(camera_transform, trigger.pointer_location.position));
            let index = (world_position / TILE_SIZE).floor().as_ivec2();

            coordinates_ui.0 = format!("({}, {})", index.x, index.y);

            // Start selection
            current_action.index_start = Some(index);
        }

        PointerButton::Secondary => {
            // Cancel selection
            current_action.index_start = None;
        }

        _ => {}
    }
}

pub fn terrain_pointer_up(
    trigger: Trigger<Pointer<Released>>,
    mut commands: Commands,
    mut current_action: ResMut<CurrentAction>,
    mut dwellers_selected: ResMut<DwellersSelected>,
    mut tilemap_data: ResMut<TilemapData>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_tasks: Query<(Entity, &Task)>,
    q_mobs: Query<(Entity, &Transform), With<Mob>>,
    mut q_dwellers: Query<(Entity, &mut Dweller, &Transform)>,
) {
    let (camera, camera_transform) = extract_ok!(q_camera.single());
    let world_position = extract_ok!(
        camera.viewport_to_world_2d(camera_transform, trigger.pointer_location.position)
    );
    let index = (world_position / TILE_SIZE).floor().as_ivec2();

    if matches!(trigger.button, PointerButton::Primary) {
        // Confirm selection
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
                                | TaskKind::Attack
                                | TaskKind::Workstation { .. },
                            )
                            | (TaskKind::Attack, _) => false,
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
                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data),
                                TaskNeeds::Nothing,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Digging task at {index:?}");
                        }

                        TaskKind::Smoothen => {
                            let task = Task::new(index, *task_kind, dweller, &tilemap_data);

                            if !task.reachable_positions.is_empty() {
                                commands.spawn(TaskBundle::new(task, TaskNeeds::Nothing));

                                max_tasks = max_tasks.saturating_sub(1);
                                debug!("Smoothening task at {index:?}");
                            }
                        }

                        TaskKind::Harvest => {
                            let needs = match tile.object {
                                Some(ObjectId::WheatPlant) => TaskNeeds::EmptyHands,
                                _ => TaskNeeds::Nothing,
                            };

                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data),
                                needs,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Harvesting task at {index:?}");
                        }

                        TaskKind::Pickup => {
                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data),
                                TaskNeeds::EmptyHands,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Picking up task at {index:?}");
                        }

                        TaskKind::Attack => {
                            if let Some((entity_mob, _)) =
                                q_mobs.iter().find(|(_, mob_transform)| {
                                    mob_transform.translation.distance(
                                        Vec3::new(index.x as f32, index.y as f32, 0.) * TILE_SIZE,
                                    ) < TILE_SIZE
                                })
                            {
                                commands.entity(entity_mob).insert(children![
                                    TaskBundle::new_as_child(
                                        Task::new(index, *task_kind, dweller, &tilemap_data)
                                            .with_priority(1),
                                        TaskNeeds::Nothing,
                                    )
                                ]);

                                max_tasks = max_tasks.saturating_sub(1);
                                debug!("Attacking task at {index:?}");
                            }
                        }

                        TaskKind::Fish => {
                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data),
                                TaskNeeds::Nothing,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Fishing task at {index:?}");
                        }

                        TaskKind::Stockpile => {
                            let needs = if tile.object.is_none() {
                                TaskNeeds::AnyObject
                            } else {
                                TaskNeeds::Impossible
                            };

                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data)
                                    .with_priority(-1),
                                needs,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Stockpiling task at {index:?}");
                        }

                        TaskKind::Walk => {
                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data)
                                    .with_priority(-1),
                                TaskNeeds::Nothing,
                            ));

                            max_tasks = max_tasks.saturating_sub(1);
                            debug!("Walk task at {index:?}");
                        }

                        _ => {}
                    },

                    ActionKind::TaskWithNeeds(task_kind, needs) => match task_kind {
                        TaskKind::Build { .. } => {
                            commands.spawn(TaskBundle::new(
                                Task::new(index, *task_kind, dweller, &tilemap_data),
                                needs.clone(),
                            ));

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

                            // stop dweller from moving towards this task
                            if let Some(dweller) = task.dweller {
                                if let Ok((_, mut dweller, _)) = q_dwellers.get_mut(dweller) {
                                    dweller.move_queue = Vec::new();
                                }
                            }

                            // if we are cancelling a Stockpile or Workstation task, mark object for pickup (if not already marked)
                            if matches!(
                                task.kind,
                                TaskKind::Stockpile | TaskKind::Workstation { .. }
                            ) && TaskKind::Pickup.is_valid_on_tile(tile)
                                && !q_tasks.iter().any(|(_, task)| {
                                    task.kind == TaskKind::Pickup && task.pos == index
                                })
                            {
                                commands.spawn(TaskBundle::new(
                                    Task::new(index, TaskKind::Pickup, dweller, &tilemap_data),
                                    TaskNeeds::EmptyHands,
                                ));

                                debug!("Cancelling {task:?} at {index:?} and marking object for pickup");
                            } else {
                                debug!("Cancelling {task:?} at {index:?}");
                            }
                        }
                    }

                    ActionKind::Select => {
                        // if single click on workstation, open workstation ui
                        if index_min == index_max {
                            if let Some(entity) = q_tasks.iter().find_map(|(entity, task)| {
                                if task.pos == index {
                                    Some(entity)
                                } else {
                                    None
                                }
                            }) {
                                commands.trigger_targets(OpenWorkstationUi, entity);
                                break;
                            }
                        }

                        // else select dwellers
                        for (entity, _, transform) in &q_dwellers {
                            if index == transform_to_index(transform) {
                                dwellers_selected.add(entity);
                            }
                        }
                    }

                    ActionKind::DebugBuild(result) => {
                        if let Some(tile) = tilemap_data.get(index) {
                            match result {
                                BuildResult::Object(object_id) => {
                                    tilemap_data.set(index, tile.id.with(*object_id));
                                }
                                BuildResult::Tile(tile_id) => {
                                    tilemap_data.set(index, tile_id.place());
                                }
                            }
                        }
                    }

                    ActionKind::DebugSpawn(mob_id) => {
                        commands.spawn(MobBundle::new(*mob_id, index));
                    }
                }
            }
        }

        current_action.index_start = None;
    }
}

pub fn terrain_draw_selection(
    mut gizmos: Gizmos,
    current_action: Res<CurrentAction>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if let Some(index_start) = current_action.index_start {
        let (camera, camera_transform) = extract_ok!(q_camera.single());
        let window = extract_ok!(q_windows.single());
        let cursor_position = extract_some!(window.cursor_position());
        let world_position =
            extract_ok!(camera.viewport_to_world_2d(camera_transform, cursor_position));
        let index = (world_position / TILE_SIZE).floor().as_ivec2();

        let from = Vec2::new(index_start.x as f32, index_start.y as f32) * TILE_SIZE;
        let to = Vec2::new(index.x as f32, index.y as f32) * TILE_SIZE;

        let center = (from + to) / 2. + TILE_SIZE / 2.;
        let size = (to - from).abs() + TILE_SIZE / 2.;

        gizmos.rect_2d(center, size, Color::WHITE);
    }
}
