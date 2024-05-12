struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0, 1);
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