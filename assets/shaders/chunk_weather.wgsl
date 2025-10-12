#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import "shaders/noise.wgsl"::smoothNoise

const CLOUD_SPEED: f32 = 0.5;
const CLOUD_SCALE: f32 = 200.0;
const CLOUD_DENSITY: f32 = 1.1; // lower = denser clouds
const CLOUD_SPARSITY: f32 = 0.7; // higher = fewer clouds, more gaps

struct ChunkWeatherMaterial {
    seed: f32,
    wind: vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ChunkWeatherMaterial;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.world_position.xy;

    // cloud alpha
    let clouds = clouds_noise(p);

    // offset sample for shadow projection
    let clouds_shadow = clouds_noise(p + vec2(100.0)); // TODO: use sun direction

    if clouds < 0.01 && clouds_shadow > clouds {
        return vec4<f32>(vec3(0.0), clouds_shadow * 4.0);
    } else {
        return vec4<f32>(vec3(1.0), clouds);
    }
}

fn clouds_noise(p: vec2<f32>) -> f32 {
    let speed = CLOUD_SPEED * material.wind;
    let motion = p / CLOUD_SCALE + speed + material.seed;

    // multiple octaves of noise
    var n = 0.0;
    n += smoothNoise(motion * 0.5) * 0.6;
    n += smoothNoise(motion * 1.0) * 0.3;
    n += smoothNoise(motion * 2.0) * 0.1;

    let biased = n - CLOUD_SPARSITY;
    let clouds = pow(max(biased, 0.0), CLOUD_DENSITY);
    return clouds;
}