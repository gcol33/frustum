// Text shader for Frustum rendering
// Renders text labels as billboarded textured quads
// Each character is a quad with UV coordinates into the font atlas

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_right: vec4<f32>,  // Camera right vector (xyz), text_scale (w)
    camera_up: vec4<f32>,     // Camera up vector (xyz), unused (w)
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var font_texture: texture_2d<f32>;

@group(0) @binding(2)
var font_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,  // Anchor position in world space
    @location(1) offset: vec2<f32>,    // Local offset from anchor (for quad corner)
    @location(2) uv: vec2<f32>,        // Texture coordinates
    @location(3) color: vec3<f32>,     // Text color
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let right = uniforms.camera_right.xyz;
    let up = uniforms.camera_up.xyz;
    let scale = uniforms.camera_right.w;

    // Billboard: offset in camera space
    let world_pos = in.position
        + right * in.offset.x * scale
        + up * in.offset.y * scale;

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(font_texture, font_sampler, in.uv);

    // Font atlas has white glyphs on transparent background
    // Use alpha for transparency, multiply RGB by text color
    if (tex_color.a < 0.1) {
        discard;
    }

    return vec4<f32>(in.color * tex_color.rgb, tex_color.a);
}
