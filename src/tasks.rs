use bevy::prelude::*;

#[derive(Component)]
pub enum Task {
    Dig(IVec2),
}
