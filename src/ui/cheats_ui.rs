use bevy::prelude::*;

use crate::{
    actions::ActionKind,
    data::{MobId, ObjectId, TileId},
    locale::Locale,
    tasks::BuildResult,
    ui::{get_observer_action_button, UiButton, UiWindow},
};

pub fn spawn_cheats_ui(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_windows: Query<Entity, With<UiWindow>>,
    asset_server: Res<AssetServer>,
    locale: Res<Locale>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        if let Some(window) = q_windows.iter().next() {
            commands.entity(window).despawn();
        } else {
            commands.spawn(UiWindow).with_children(|c| {
                c.spawn(Node {
                    display: Display::Grid,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    grid_auto_rows: GridTrack::min_content(),
                    grid_template_columns: vec![GridTrack::min_content(); 9],
                    ..default()
                })
                .with_children(|c| {
                    let results = ObjectId::ALL
                        .iter()
                        .map(|&object| BuildResult::Object(object))
                        .chain(TileId::ALL.iter().map(|&tile| BuildResult::Tile(tile)));

                    for result in results {
                        let label = match result {
                            BuildResult::Object(obj) => locale.t(&format!("object.{obj:?}")),
                            BuildResult::Tile(tile) => locale.t(&format!("tile.{tile:?}")),
                        };
                        c.spawn(UiButton)
                            .with_child(Text::new(label))
                            .with_child(ImageNode::new(asset_server.load(result.sprite_path())))
                            .observe(get_observer_action_button(ActionKind::DebugBuild(result)));
                    }

                    for mob in MobId::ALL {
                        let label = locale.t(&format!("mob.{mob:?}"));
                        c.spawn(UiButton)
                            .with_child(Text::new(label))
                            .with_child(ImageNode::new(asset_server.load(mob.data().sprite_path())))
                            .observe(get_observer_action_button(ActionKind::DebugSpawn(*mob)));
                    }
                });
            });
        }
    }
}
