use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;

use crate::{
    actions::*, camera::*, dwellers::*, mobs::*, save_load::*, tasks::*, terrain::*, tiles::*,
    ui::*,
};

mod actions;
mod camera;
mod dwellers;
mod mobs;
mod save_load;
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
            SaveLoadPlugin,
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
                // UI
                update_camera,
                update_ui,
                keyboard_current_action,
                focus_any_dweller,
                click_terrain,
                // Game logic
                (update_dwellers, update_mobs).run_if(on_timer(Duration::from_millis(200))),
                update_dwellers_movement,
                update_mobs_movement,
                update_unreachable_tasks,
                update_pickups,
                event_task_completion,
            ),
        )
        .run();
}
