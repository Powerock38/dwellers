use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::save_load::SaveName;

const DAY_LENGTH_SECS: f32 = 300.0;
const WIND_CHANGE_CHANCE: f64 = 0.0001;

#[derive(Resource, Default)]
pub struct Weather {
    pub elapsed_secs: f32,
    pub wind: Vec2,
    pub target_wind: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkWeatherMaterial {
    #[uniform(0)]
    pub seed: u32,
    #[uniform(0)]
    pub time_of_day: f32, // 0.0 (midnight) 0.5 (noon)
    #[uniform(0)]
    pub wind: Vec2,
}

impl ChunkWeatherMaterial {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            time_of_day: 0.0,
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

    if rng.random_bool(WIND_CHANGE_CHANCE) {
        weather.target_wind = random_wind(save_name.seed());
    }

    weather.elapsed_secs += time.delta_secs();

    let lerp_speed = 0.01 * time.delta_secs();
    weather.wind = weather.wind.lerp(weather.target_wind, lerp_speed);

    // start in the morning
    let time_of_day = (weather.elapsed_secs + DAY_LENGTH_SECS / 4.0) / DAY_LENGTH_SECS % 1.0;

    for material in query.iter() {
        let material = materials.get_mut(material).unwrap();
        material.wind = weather.wind * weather.elapsed_secs;
        material.time_of_day = time_of_day;
    }
}
