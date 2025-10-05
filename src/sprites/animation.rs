use bevy::prelude::*;

#[derive(Component)]
pub struct TakingDamage {
    time: Timer,
}

impl TakingDamage {
    pub fn new() -> Self {
        TakingDamage {
            time: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

pub fn update_taking_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TakingDamage, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut taking_damage, mut sprite, mut transform) in &mut query {
        if taking_damage.time.tick(time.delta()).just_finished() {
            sprite.color = Color::WHITE;
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<TakingDamage>();
        } else {
            sprite.color = Srgba::new(1.0, 0.0, 0.0, 1.0 - taking_damage.time.fraction()).into();
            transform.scale = Vec3::splat(0.9 + taking_damage.time.fraction() * 0.2);
        }
    }
}
