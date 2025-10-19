#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import "shaders/noise.wgsl"::smoothNoise

const NIGHT_INTENSITY: f32 = 0.9;

const CLOUD_SPEED: f32 = 0.2;
const CLOUD_SCALE: f32 = 200.0;
const CLOUD_SPARSITY: f32 = 0.6; // higher = fewer clouds, more gaps
const CLOUD_DENSITY: f32 = 0.4; // lower = denser clouds
const CLOUD_SHADOW_LENGTH: f32 = 100.0;
const CLOUD_SHADOW_MIN_OPACITY: f32 = 0.45;

const PI: f32 = 3.141592653589793;

struct ChunkWeatherMaterial {
    seed: f32,
    time_of_day: f32,
    wind: vec2<f32>,
    cloud_opacity: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ChunkWeatherMaterial;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.world_position.xy;
    let clouds = material.cloud_opacity * clouds_noise(p);

    let light_angle = material.time_of_day * PI;
    let light_dir = vec2(cos(light_angle), sin(light_angle));
    let clouds_shadow_distance = CLOUD_SHADOW_LENGTH * sin(light_angle);
    let clouds_shadow = clouds_noise(p + light_dir * clouds_shadow_distance);

    let shadow_blend = smoothstep(0.1, 0.01, clouds);
    let cloud_alpha = clouds + clouds_shadow * shadow_blend * max(material.cloud_opacity, CLOUD_SHADOW_MIN_OPACITY);
    let clouds_color = vec4(vec3f(clouds), cloud_alpha);

    let night_cycle = (cos((material.time_of_day - 0.5) * 2.0 * PI) * 0.5) + 0.5;
    let night_alpha = (1.0 - night_cycle) * NIGHT_INTENSITY;
    let night_color = vec4(vec3(0.0), 1.0);

    return mix(clouds_color, night_color, night_alpha);
}

fn clouds_noise(p: vec2<f32>) -> f32 {
    let speed = CLOUD_SPEED * material.wind;
    let motion = p / CLOUD_SCALE + speed + material.seed;

    // Domain warp
    let warp = vec2(
        smoothNoise(motion * 0.4),
        smoothNoise(motion * 0.4 + vec2(100.0, 100.0))
    );
    let warped = motion + warp * 1.5;

    // Octaves on warped coordinates
    var n = 0.0;
    n += smoothNoise(warped * 0.5) * 0.5;
    n += smoothNoise(warped * 1.0) * 0.25;
    n += smoothNoise(warped * 2.0) * 0.125;
    n += smoothNoise(warped * 4.0) * 0.0625;

    let biased = n - CLOUD_SPARSITY;
    let clouds = pow(max(biased, 0.0), CLOUD_DENSITY);
    return clouds;
}
