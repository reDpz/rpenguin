var<private> VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
);

struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_position: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    // instance: ParticleInstanceInput,
) -> VertexOutput {
    var out: VertexOutput;


    out.vert_position = VERTICES[in.index];
    out.clip_position = vec4<f32>(VERTICES[in.index].x, VERTICES[in.index].y, 0.0, 0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
