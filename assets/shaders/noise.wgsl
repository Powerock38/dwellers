
#import bevy_sprite::mesh2d_view_bindings::globals

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

fn fractalNoise(p: vec2<f32>) -> f32 {
    var x: f32 = 0.0;
    x += smoothNoise(p);
    x += smoothNoise(p * 2.0) / 2.0;
    x += smoothNoise(p * 4.0) / 4.0;
    x += smoothNoise(p * 8.0) / 8.0;
    x += smoothNoise(p * 16.0) / 16.0;
    x /= 1.0 + 1.0 / 2.0 + 1.0 / 4.0 + 1.0 / 8.0 + 1.0 / 16.0;
    return x;
}

fn movingNoise(p: vec2<f32>, speed: f32, seed: f32) -> f32 {
    let x = fractalNoise(p + globals.time * speed + seed);
    let y = fractalNoise(p - globals.time * speed + seed);
    return fractalNoise(p + vec2<f32>(x, y));
}

fn nestedMovingNoise(p: vec2<f32>, speed: f32, seed: f32) -> f32 {
    let x = movingNoise(p, speed, seed);
    let y = movingNoise(p, speed, 2.0 * seed);
    return movingNoise(p + vec2<f32>(x, y), speed, seed);
}