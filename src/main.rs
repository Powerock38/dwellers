use std::time::Duration;

use actions::{click_terrain, keyboard_current_action};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;
use camera::{update_camera, CameraControl};
use dwellers::{spawn_dwellers, update_dwellers, update_dwellers_movement};
use tasks::{update_tasks_assignements, update_unreachable_tasks};
use terrain::spawn_terrain;
use tiles::{event_mine_tile, event_smoothen_tile, MineTile, SmoothenTile};
use ui::{spawn_ui, update_ui};

mod actions;
mod camera;
mod dwellers;
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
        .add_event::<MineTile>()
        .add_event::<SmoothenTile>()
        .add_systems(Startup, (spawn_terrain, spawn_dwellers, spawn_ui))
        .add_systems(
            Update,
            (
                update_camera,
                update_ui,
                keyboard_current_action,
                click_terrain,
                (
                    update_tasks_assignements.before(update_dwellers),
                    update_dwellers,
                )
                    .run_if(on_timer(Duration::from_millis(200))),
                update_dwellers_movement,
                update_unreachable_tasks,
                event_mine_tile,
                event_smoothen_tile,
            ),
        )
        .run();
}
