use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use crate::{dwellers::Dweller, CHUNK_SIZE, TILE_SIZE};

const CAMERA_KEYBOARD_SPEED: f32 = 3.0;

#[derive(Resource)]
pub struct CameraControl {
    pub target_pos: Vec2,
    pub target_scale: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            target_pos: Vec2::splat(CHUNK_SIZE as f32 * 0.5 * TILE_SIZE),
            target_scale: 1.,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn update_camera(
    query: Single<(&mut Transform, &mut Projection)>,
    input_keyboard: Res<ButtonInput<KeyCode>>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut event_wheel: EventReader<MouseWheel>,
    mut event_move: EventReader<MouseMotion>,
    time: Res<Time>,
    mut control: ResMut<CameraControl>,
) {
    let (mut transform, mut projection) = query.into_inner();

    let Projection::Orthographic(projection) = projection.as_mut() else {
        return;
    };

    if input_mouse.pressed(MouseButton::Right) {
        for ev in event_move.read() {
            control.target_pos +=
                projection.scale * ev.delta * time.delta_secs() * 200. * Vec2::new(-1., 1.);
        }
    } else {
        let mut step = 270. * time.delta_secs();
        if input_keyboard.pressed(KeyCode::ShiftLeft) {
            step *= 2.;
        }

        let mut x = 0;
        if input_keyboard.pressed(KeyCode::KeyD) {
            x += 1;
        }
        if input_keyboard.pressed(KeyCode::KeyA) {
            x -= 1;
        }
        control.target_pos += x as f32 * step * Vec2::X * CAMERA_KEYBOARD_SPEED;

        let mut y = 0;
        if input_keyboard.pressed(KeyCode::KeyW) {
            y += 1;
        }
        if input_keyboard.pressed(KeyCode::KeyS) {
            y -= 1;
        }
        control.target_pos += y as f32 * step * Vec2::Y * CAMERA_KEYBOARD_SPEED;
    }

    let target = control.target_pos.extend(0.);
    if transform.translation.distance_squared(target) > 0.01 {
        transform.translation = transform.translation.lerp(target, 40. * time.delta_secs());
    }

    for ev in event_wheel.read() {
        control.target_scale -= ev.y * 0.05;
    }
    control.target_scale = control.target_scale.clamp(0.01, 2.0);

    if (projection.scale - control.target_scale).abs() > 0.01 {
        projection.scale = projection.scale
            + ((control.target_scale - projection.scale) * 20. * time.delta_secs());
    }
    event_move.clear();
}

pub fn focus_any_dweller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_dwellers: Query<&Transform, With<Dweller>>,
    q_new_dweller: Query<&Transform, Added<Dweller>>,
    mut control: ResMut<CameraControl>,
) {
    let mut transform = q_new_dweller.iter().next();
    if keyboard_input.just_pressed(KeyCode::KeyQ) || transform.is_some() {
        if transform.is_none() {
            transform = q_dwellers.iter().next();
        }

        if let Some(transform) = transform {
            info!("Focusing on dweller {:?}", transform.translation.truncate());
            control.target_pos = transform.translation.truncate();
            control.target_scale = 0.5;
        }
    }
}
