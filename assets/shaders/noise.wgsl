#import bevy_sprite::mesh2d_view_bindings::globals

// Moving fractal noise

fn random(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    return random(p.x + p.y * 10000.0);
}

fn sw(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(floor(p.x), floor(p.y));
}

fn se(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(ceil(p.x), floor(p.y));
}

fn nw(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(floor(p.x), ceil(p.y));
}

fn ne(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(ceil(p.x), ceil(p.y));
}

fn smoothNoise(p: vec2<f32>) -> f32 {
    let interp = vec2<f32>(smoothstep(0.0, 1.0, fract(p.x)), smoothstep(0.0, 1.0, fract(p.y)));
    let s = mix(noise(sw(p)), noise(se(p)), interp.x);
    let n = mix(noise(nw(p)), noise(ne(p)), interp.x);
    return mix(s, n, interp.y);
}
