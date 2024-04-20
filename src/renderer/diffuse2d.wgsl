// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct SpriteUniform {
    //buffer_dimensions: vec2<f32>,
    render_target_dimensions: vec2<f32>,
    position: vec2<f32>,
    dimensions: vec2<f32>,
    rotation_center: vec2<f32>,
    rotation: mat2x2<f32>,
}

@group(1) @binding(0)
var<uniform> sprite: SpriteUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let scaled = model.position * sprite.dimensions;
    let rotated = (scaled - sprite.rotation_center) * sprite.rotation + sprite.rotation_center;
    let translated = rotated + sprite.position;
    let flipped_for_renderer = vec2<f32>(translated.x, -translated.y);
    let scaled_to_renderer= (flipped_for_renderer / sprite.render_target_dimensions) * 2.0;
    let translated_to_render_coords = scaled_to_renderer + vec2<f32>(-1.0, 1.0);

    out.clip_position = vec4<f32>(translated_to_render_coords, 0.0, 1.0);

    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

