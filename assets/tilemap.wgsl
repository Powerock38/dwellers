#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct TilemapMaterial {
    brightness: f32,
};

@group(3) @binding(0)
var<uniform> material: TilemapMaterial;

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);

    // let hsv = vec3(abs(sin(in.uv.x)), abs(sin(in.uv.y)), 1.0);
    // return vec4((color.rgb + hsv2rgb(hsv)) * material.brightness, color.a);

    // return vec4(sdCircle(in.uv.xy - 0.5, 0.1), 0.5, 1.0, 1.0);

    return vec4(f32(in.tile_id) / 40., 0.5, 1.0, 1.0);
}