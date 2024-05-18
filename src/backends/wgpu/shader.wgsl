struct Uniform {
    screen_size: vec2<i32>,
};

@group(0) @binding(0) var<uniform> uni: Uniform;
//@group(1) @binding(0) var tex: texture_2d<f32>;
//@group(1) @binding(1) var tex_sampler: sampler;

struct VertexInput {
    @location(0) screen_xy: vec2<i32>,
    @location(1) texture_xy: vec2<i32>,
    @location(2) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec4<f32>,
    @location(1) colour: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = coord_conv(in.screen_xy, uni.screen_size);

    // Negative texture coordinates = no use of texture in this vertex
    let use_texture = 0 <= in.texture_xy.x && 0 < in.texture_xy.y;
    if use_texture {
        out.texture_position = coord_conv(in.texture_xy, uni.screen_size);
    } else {
        out.texture_position = vec4<f32>(-1.0);
    }

    out.colour = in.colour;

    return out;
}

fn coord_conv(coord: vec2<i32>, canvas_size: vec2<i32>) -> vec4<f32> {
    let can = vec2<f32>(canvas_size);
    var pos = vec2<f32>(coord);

    // Normalise to (0.0, 2.0)
    pos = (pos / (can * vec2<f32>(0.5)));

    // Shift to (-1.0, 1.0)
    pos = pos - vec2<f32>(1.0);

    // Invert Y coordinates
    pos = pos * vec2<f32>(1.0, -1.0);

    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return srgb_conv(in.colour);
}

fn srgb_conv(colour: vec4<f32>) -> vec4<f32> {
    var comp = colour / vec4<f32>(255.0);
    comp = comp + vec4<f32>(0.055, 0.055, 0.055, 0.0);
    comp = comp / vec4<f32>(1.055, 1.055, 1.055, 1.0);
    comp = pow(comp, vec4<f32>(2.4, 2.4, 2.4, 1.0));
    return comp;
}