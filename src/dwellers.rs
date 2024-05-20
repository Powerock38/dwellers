use bevy::prelude::*;
use rand::Rng;

use crate::{
    extract_ok, extract_some,
    terrain::{TilemapData, TILE_SIZE},
};

#[derive(Component)]
pub struct Dweller {
    name: String,
    age: u32,
    health: u32,
}

//TODO: task queue

#[derive(Bundle)]
pub struct DwellerBundle {
    dweller: Dweller,
    sprite: SpriteBundle,
}

pub fn spawn_dwellers(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DwellerBundle {
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/dweller.png"),
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::TopLeft,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        },
        dweller: Dweller {
            name: "Alice".to_string(),
            age: 30,
            health: 100,
        },
    });
}

pub fn update_dwellers(
    mut q_dwellers: Query<(&Dweller, &mut Transform)>,
    q_tilemap_data: Query<&TilemapData>,
) {
    for (dweller, mut transform) in q_dwellers.iter_mut() {
        // Wander around

        let mut index = IVec2::new(
            (transform.translation.x / TILE_SIZE) as i32,
            (transform.translation.y / TILE_SIZE) as i32,
        );

        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.5) {
            index.x += rng.gen_range(-1..=1);
        } else {
            index.y += rng.gen_range(-1..=1);
        }

        let tilemap_data = extract_ok!(q_tilemap_data.get_single());
        let tiledata = extract_some!(tilemap_data.0.get(index));

        if !tiledata.wall {
            transform.translation.x = index.x as f32 * TILE_SIZE;
            transform.translation.y = index.y as f32 * TILE_SIZE;
        }

        //TODO: check for Tasks
    }
}
