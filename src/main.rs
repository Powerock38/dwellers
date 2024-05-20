use std::time::Duration;

use actions::{click_terrain, keyboard_current_action};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::{update_camera, CameraControl};
use dwellers::{spawn_dwellers, update_dwellers, update_dwellers_next_move};
use tasks::update_pathfinding_tasks;
use terrain::{event_mine_tile, spawn_terrain, update_path_tilemaps};

mod actions;
mod camera;
mod dwellers;
mod tasks;
mod terrain;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::default(),
            EntiTilesPlugin,
        ))
        .init_resource::<CameraControl>()
        .add_systems(Startup, (spawn_terrain, spawn_dwellers))
        .add_systems(
            Update,
            (
                update_camera,
                keyboard_current_action,
                click_terrain,
                (
                    update_dwellers,
                    update_dwellers_next_move.after(update_dwellers),
                )
                    .run_if(on_timer(Duration::from_millis(100))),
                update_pathfinding_tasks,
                event_mine_tile,
                update_path_tilemaps,
            ),
        )
        .run();
}
