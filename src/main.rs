use bevy::{prelude::*, window::PresentMode};
use bevy_entitiles::EntiTilesPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::{update_camera, CameraControl};
use digging::dig_terrain;
use terrain::spawn_terrain;

mod camera;
mod digging;
mod terrain;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            WorldInspectorPlugin::default(),
            EntiTilesPlugin,
        ))
        .init_resource::<CameraControl>()
        .add_systems(Startup, spawn_terrain)
        .add_systems(Update, (update_camera, dig_terrain))
        .run();
}
