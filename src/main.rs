use std::time::Duration;

use actions::{click_terrain, keyboard_current_action};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::{algorithm::pathfinding::pathfinding_scheduler, EntiTilesPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::{update_camera, CameraControl};
use dwellers::{spawn_dwellers, update_dwellers, update_pathfinding_tasks};
use tasks::update_path_tilemaps;
use terrain::spawn_terrain;
use tiles::{event_mine_tile, event_smoothen_tile, MineTile, SmoothenTile};

mod actions;
mod camera;
mod dwellers;
mod tasks;
mod terrain;
mod tiles;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::default(),
            EntiTilesPlugin,
        ))
        .init_resource::<CameraControl>()
        .add_event::<MineTile>()
        .add_event::<SmoothenTile>()
        .add_systems(Startup, (spawn_terrain, spawn_dwellers))
        .add_systems(
            Update,
            (
                update_camera,
                keyboard_current_action,
                click_terrain,
                (
                    update_dwellers,
                    update_pathfinding_tasks.after(update_dwellers),
                )
                    .run_if(on_timer(Duration::from_millis(200))),
                update_path_tilemaps.before(pathfinding_scheduler),
                event_mine_tile,
                event_smoothen_tile,
            ),
        )
        .run();
}
