// Vertex shader

struct CameraUniform {
    proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_position: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    // @builtin() in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.proj * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    out.vert_position = in.position;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_squared = (in.vert_position.x * in.vert_position.x) + (in.vert_position.y * in.vert_position.y);

    if dist_squared > 0.2 {
        discard;
    }


    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

@fragment
fn fs_white(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

// this is the distance squared
fn distance_v2_sq(one: vec2<f32>, two: vec2<f32>) -> f32 {
    let delta_x = one.x - two.x;
    let delta_y = one.y - two.y;
    return (delta_x * delta_x + delta_y * delta_y);
}
