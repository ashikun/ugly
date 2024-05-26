struct Uniform {
    screen_size : vec2<i32>,  // Physical screen size (pixels)
    ignored     : i32,        // Padding
    scale_factor: f32,        // Screen scaling factor (x)
};

@group(0) @binding(0) var<uniform> uni: Uniform;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var tex_sampler: sampler;

struct VertexInput {
    @location(0) screen_xy: vec2<i32>,
    @location(1) texture_xy: vec2<i32>,
    @location(2) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
    @location(1) colour: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = coord_conv(in.screen_xy);
    out.texture_position = tex_coord_conv(in.texture_xy);
    out.colour = in.colour;

    return out;
}

/// Convert a screen coordinate (XY) to a clip-space coordinate (XYZW)
fn coord_conv(in: vec2<i32>) -> vec4<f32> {
    let screen = vec2<f32>(uni.screen_size);

    var out = vec2<f32>(in);
    out    *= vec2<f32>(uni.scale_factor);  // Convert logical to physical screen coordinates
    out.y   = (screen.y - 1) - out.y;       // Invert Y coordinates, keeping within (0..screen.y)
    out    /= (screen * vec2<f32>(0.5));    // Normalise to (0.0, 2.0)
    out    -= vec2<f32>(1.0);               // Shift to (-1.0, 1.0)

    return vec4<f32>(out, 0.0, 1.0);
}

fn tex_coord_conv(in: vec2<i32>) -> vec2<f32> {
    var out = vec2<f32>(in);
    out /= vec2<f32>(textureDimensions(tex));  // Normalise to (0.0, 1.0)
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var col = in.colour;

    // Out of bounds texture coordinates is our idiom for disabling texture sampling
    // TODO: find a more robust way of doing this
    let x_in_bounds = 0 <= in.texture_position.x && in.texture_position.x <= 1;
    let y_in_bounds = 0 <= in.texture_position.y && in.texture_position.y <= 1;
    if x_in_bounds && y_in_bounds {
        col *= textureSample(tex, tex_sampler, in.texture_position);
    }

    return srgb_conv(col);
}

fn srgb_conv(colour: vec4<f32>) -> vec4<f32> {
    var comp = colour / vec4<f32>(255.0);
    comp = comp + vec4<f32>(0.055, 0.055, 0.055, 0.0);
    comp = comp / vec4<f32>(1.055, 1.055, 1.055, 1.0);
    comp = pow(comp, vec4<f32>(2.4, 2.4, 2.4, 1.0));
    return comp;
}