use std::time::Duration;

use actions::click_terrain;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::{update_camera, CameraControl};
use dwellers::{spawn_dwellers, update_dwellers};
use terrain::spawn_terrain;

mod actions;
mod camera;
mod dwellers;
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
                click_terrain,
                update_dwellers.run_if(on_timer(Duration::from_millis(100))),
            ),
        )
        .run();
}
