use bevy::prelude::*;
pub use save_load_assets::*;
pub use save_load_systems::*;
pub use save_load_ui::*;

use crate::{
    data::ObjectId, BuildResult, Dweller, Mob, Task, TaskKind, TaskNeeds, TileData, TileKind,
};

mod save_load_assets;
mod save_load_systems;
mod save_load_ui;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                save_world,
                load_world,
                spawn_load_save_ui,
                scan_sprite_loaders,
            ),
        )
        .insert_resource(SaveName("new_world".to_string()))
        .register_type::<SaveName>()
        .register_type::<Dweller>()
        .register_type::<ObjectId>()
        .register_type::<Option<ObjectId>>()
        .register_type::<Vec<ObjectId>>()
        .register_type::<TileKind>()
        .register_type::<TileData>()
        .register_type::<Mob>()
        .register_type::<Task>()
        .register_type::<Option<Entity>>()
        .register_type::<TaskKind>()
        .register_type::<BuildResult>()
        .register_type::<TaskNeeds>()
        .register_type::<Vec<IVec2>>()
        .register_type::<SpriteLoader>();
    }
}
