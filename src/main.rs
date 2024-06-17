use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_entitiles::EntiTilesPlugin;

use crate::{
    actions::*, camera::*, dwellers::*, mobs::*, save_load::*, state::*, tasks::*, terrain::*,
    tilemap::*, tiles::*, ui::*,
};

mod actions;
mod camera;
mod data;
mod dwellers;
mod mobs;
mod save_load;
mod state;
mod tasks;
mod terrain;
mod tilemap;
mod tiles;
mod ui;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    filter: "wgpu=error,naga=warn,dungeons=debug".into(),
                    ..default()
                }),
            EntiTilesPlugin,
            // bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            SaveLoadPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<CameraControl>()
        .add_event::<TaskCompletionEvent>()
        .configure_sets(Update, GameplaySet.run_if(in_state(GameState::Running)))
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
                update_ui_buttons,
                update_camera,
                toggle_state,
                (
                    // Game UI
                    keyboard_current_action,
                    focus_any_dweller,
                    click_terrain,
                    // Game logic
                    (update_dwellers, update_mobs).run_if(on_timer(Duration::from_millis(200))),
                    (update_terrain).run_if(on_timer(Duration::from_millis(800))),
                    update_dwellers_movement,
                    update_mobs_movement,
                    update_unreachable_tasks,
                    update_pickups,
                    event_task_completion,
                )
                    .in_set(GameplaySet),
            ),
        )
        .init_state::<GameState>()
        .run();
}
