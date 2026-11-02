// Basic shader for Frustum rendering
// Transforms vertices and applies Lambertian shading (meshes only)

struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,     // xyz = direction toward light, w = intensity
    light_config: vec4<f32>,  // x = enabled (0 or 1), yzw unused
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// One row per colormap; sampled along u at the interpolated scalar
@group(0) @binding(1)
var cmap_tex: texture_2d<f32>;
@group(0) @binding(2)
var cmap_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) cmap: vec2<f32>,      // x = scalar in [0,1], y = colormap row (< 0: use color)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) cmap: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.color = in.color;
    out.cmap = in.cmap;
    return out;
}

// Base colour: the colormap sampled at the interpolated scalar, or the
// interpolated vertex colour where no colormap applies
fn base_color(in: VertexOutput) -> vec3<f32> {
    let rows = f32(textureDimensions(cmap_tex).y);
    let uv = vec2<f32>(in.cmap.x, (in.cmap.y + 0.5) / rows);
    let mapped = textureSample(cmap_tex, cmap_sampler, uv).rgb;
    let use_cmap = in.cmap.y >= 0.0 && in.cmap.x >= 0.0;
    return select(in.color, mapped, use_cmap);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let enabled = uniforms.light_config.x > 0.5;
    let color = base_color(in);

    // If lighting disabled, return flat color
    if !enabled {
        return vec4<f32>(color, 1.0);
    }

    // Normalize the interpolated normal
    let normal = normalize(in.world_normal);

    // Lambertian diffuse: max(0, N · L) * intensity
    let light_dir = uniforms.light_dir.xyz;
    let intensity = uniforms.light_dir.w;
    let n_dot_l = max(dot(normal, light_dir), 0.0);

    // Apply lighting: base_color * diffuse_term
    // Clamp to avoid over-brightening
    let lit_color = color * min(n_dot_l * intensity, 1.0);

    return vec4<f32>(lit_color, 1.0);
}
