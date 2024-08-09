// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>
}
// @group(0) @binding(0)
// var<uniform> camera:CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    // instance: InstanceInput,
) -> VertexOutput {

    var out: VertexOutput;

    

    // convert from world space to camera space
    // out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 0.0, 1.0);
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(0.0, 0.0, 1.0, 1.0);

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

@fragment
fn fs_white(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

// seems to be correct
fn distance_v2(one: vec2<f32>, two: vec2<f32>) -> f32 {
    let delta_x = one.x - two.x;
    let delta_y = one.y - two.y;
    return sqrt((delta_x * delta_x + delta_y * delta_y));
}
