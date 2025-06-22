use bevy::prelude::*;
pub use save_load_assets::*;
pub use save_load_systems::*;
pub use save_load_ui::*;

use crate::{Dweller, Mob, Task, TaskNeeds};

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
        .register_type::<SaveName>()
        .register_type::<Dweller>()
        .register_type::<Mob>()
        .register_type::<Task>()
        .register_type::<TaskNeeds>()
        .register_type::<SpriteLoader>();
    }
}
