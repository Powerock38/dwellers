use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::save_load::SaveName;

#[derive(Resource, Default)]
pub struct Weather {
    pub wind: Vec2,
    pub target_wind: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkWeatherMaterial {
    #[uniform(0)]
    pub seed: u32,
    #[uniform(0)]
    pub wind: Vec2,
}

impl ChunkWeatherMaterial {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            wind: Vec2::default(),
        }
    }
}

impl Material2d for ChunkWeatherMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk_weather.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub fn random_wind(seed: u32) -> Vec2 {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed as u64);
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let strength = rng.random_range(0.0..1.0);
    let wind = Vec2::new(angle.cos(), angle.sin()) * strength;
    wind.clamp_length_max(1.0)
}

pub fn update_weather(
    time: Res<Time>,
    save_name: Res<SaveName>,
    mut weather: ResMut<Weather>,
    query: Query<&MeshMaterial2d<ChunkWeatherMaterial>>,
    mut materials: ResMut<Assets<ChunkWeatherMaterial>>,
) {
    let mut rng = rand::rng();

    if weather.is_added() {
        weather.wind = random_wind(save_name.seed());
        weather.target_wind = random_wind(save_name.seed());
    }

    if rng.random_bool(0.0001) {
        weather.target_wind = random_wind(save_name.seed());
    }

    let lerp_speed = 0.01 * time.delta_secs();
    weather.wind = weather.wind.lerp(weather.target_wind, lerp_speed);

    for material in query.iter() {
        let material = materials.get_mut(material).unwrap();
        material.wind = weather.wind * time.elapsed_secs(); //TODO: save elapsed time in Weather, so it's consistent across saves
    }
}
