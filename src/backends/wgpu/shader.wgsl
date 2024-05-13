struct Uniform {
    screen_size: vec2<i32>,
};

@group(0) @binding(0) var<uniform> uni: Uniform;

struct VertexInput {
    @location(0) position: vec2<i32>,
    @location(1) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var scr = vec2<f32>(uni.screen_size);
    var pos = vec2<f32>(in.position);

    // Normalise to (0.0, 2.0)
    pos = (pos / (scr * vec2<f32>(0.5)));

    // Shift to (-1.0, 1.0)
    pos = pos - vec2<f32>(1.0);

    // Invert Y coordinates
    pos = pos * vec2<f32>(1.0, -1.0);

    out.clip_position = vec4<f32>(pos, 0, 1);
    out.colour = srgb_conv(in.colour);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.colour;
}

fn srgb_conv(colour: vec4<f32>) -> vec4<f32> {
    var comp = colour / vec4<f32>(255.0);
    comp = comp + vec4<f32>(0.055, 0.055, 0.055, 0.0);
    comp = comp / vec4<f32>(1.055, 1.055, 1.055, 1.0);
    comp = pow(comp, vec4<f32>(2.4, 2.4, 2.4, 1.0));
    return comp;
}