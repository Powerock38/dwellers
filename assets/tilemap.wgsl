#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct TilemapMaterial {
    brightness: f32,
};

@group(3) @binding(0)
var<uniform> material: TilemapMaterial;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);

    return vec4(color.rgb * material.brightness, color.a);
}