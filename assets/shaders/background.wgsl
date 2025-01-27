#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/noise.wgsl"::nestedMovingNoise

const COLOR: vec3f = vec3f(0.05, 0.02, 0.01);
const SCALE: f32 = 10.0;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let noise = nestedMovingNoise(mesh.uv * SCALE, 0.1, 0.0);

    return noise * vec4f(COLOR, 1.0);
}
