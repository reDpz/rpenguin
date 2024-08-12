// Vertex shader

var<private> VERTICES: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    // this is totally my code
    vec2<f32>(-1.7321, -1.0),
    vec2<f32>(1.7321, -1.0), // sqrt(3) ≈ 1.7321
    vec2<f32>(0.0, 2.0),
);

struct CameraUniform {
    proj: mat4x4<f32>,
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
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
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let instance_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    out.vert_position = VERTICES[in.index];
    out.clip_position = camera.proj * instance_matrix * vec4<f32>(out.vert_position.x, out.vert_position.y, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_squared = (in.vert_position.x * in.vert_position.x) + (in.vert_position.y * in.vert_position.y);

    if dist_squared > 1.0 {
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
