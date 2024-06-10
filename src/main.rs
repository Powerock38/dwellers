use std::time::Duration;

use actions::{click_terrain, keyboard_current_action};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;
use camera::{focus_any_dweller, update_camera, CameraControl};
use dwellers::{spawn_dwellers, update_dwellers, update_dwellers_movement};
use mobs::{spawn_mobs, update_mobs, update_mobs_movement};
use tasks::{event_task_completion, update_pickups, update_unreachable_tasks, TaskCompletionEvent};
use terrain::spawn_terrain;
use ui::{spawn_ui, update_ui};

mod actions;
mod camera;
mod dwellers;
mod mobs;
mod tasks;
mod terrain;
mod tiles;
mod ui;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            // bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<CameraControl>()
        /*
        .insert_resource(ChunkSaveConfig {
            path: "generated/chunk_unloading".to_string(),
            chunks_per_frame: 1,
        })
        .insert_resource(ChunkLoadConfig {
            path: "generated/chunk_unloading".to_string(),
            chunks_per_frame: 1,
        })
        */
        .add_event::<TaskCompletionEvent>()
        .add_systems(
            Startup,
            (
                spawn_terrain,
                (spawn_dwellers, spawn_mobs).after(spawn_terrain),
                spawn_ui,
            ),
        )
        .add_systems(
            Update,
            (
                update_camera,
                update_ui,
                focus_any_dweller,
                click_terrain,
                (update_dwellers, update_mobs).run_if(on_timer(Duration::from_millis(200))),
                update_dwellers_movement,
                update_mobs_movement,
                update_unreachable_tasks,
                update_pickups,
                event_task_completion,
                keyboard_current_action,
            ),
        )
        .run();
}
