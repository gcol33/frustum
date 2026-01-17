// Point shader for Frustum rendering
// Renders point clouds as billboarded quads (camera-facing)
// Uses instancing: each point is an instance, each instance draws a quad

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_right: vec4<f32>,  // Camera right vector (xyz), point_size (w)
    camera_up: vec4<f32>,     // Camera up vector (xyz), unused (w)
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,  // Point center (instance data)
    @location(1) color: vec3<f32>,     // Point color (instance data)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    // Quad corner offsets (two triangles: 0-1-2, 2-1-3)
    // UV coords for potential circular point masking
    var offsets = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),  // 0: bottom-left
        vec2<f32>( 1.0, -1.0),  // 1: bottom-right
        vec2<f32>(-1.0,  1.0),  // 2: top-left
        vec2<f32>(-1.0,  1.0),  // 3: top-left
        vec2<f32>( 1.0, -1.0),  // 4: bottom-right
        vec2<f32>( 1.0,  1.0),  // 5: top-right
    );

    let offset = offsets[vertex_index];
    let point_size = uniforms.camera_right.w;

    // Billboard: offset in camera space, then transform to clip space
    let right = uniforms.camera_right.xyz;
    let up = uniforms.camera_up.xyz;

    // Scale factor: point_size is in pixels, convert to world units
    // This is approximate - proper scaling would need projection info
    let scale = point_size * 0.005;  // Tunable factor

    let world_pos = in.position
        + right * offset.x * scale
        + up * offset.y * scale;

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = in.color;
    out.uv = offset;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Circular point: discard pixels outside unit circle
    let dist_sq = in.uv.x * in.uv.x + in.uv.y * in.uv.y;
    if (dist_sq > 1.0) {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}
