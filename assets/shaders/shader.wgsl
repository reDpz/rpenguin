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

struct ParticleInstanceInput {
    @location(5) position: vec2<f32>,
    @location(6) color: vec3<f32>,
    @location(7) radius: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) radius: f32,
}

@vertex
fn vs_main(
    in: VertexInput,
    instance: ParticleInstanceInput,
) -> VertexOutput {
    var out: VertexOutput;


    out.vert_position = VERTICES[in.index] * instance.radius;
    out.clip_position = camera.proj * vec4<f32>(out.vert_position.x + instance.position.x, out.vert_position.y + instance.position.y, 0.0, 1.0);
    out.radius = instance.radius;
    out.color = instance.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_squared = (in.vert_position.x * in.vert_position.x) + (in.vert_position.y * in.vert_position.y);

    if dist_squared > in.radius * in.radius {
        discard;
    }


    return vec4<f32>(in.color, 1.0);
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
