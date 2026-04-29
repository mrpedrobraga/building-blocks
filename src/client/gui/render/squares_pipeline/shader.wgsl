/* 
    PIXEL RENDERING SHADER

    This shader will render layers of pixel data on top of one another.
*/

/*
    Global Uniforms
    Layer Uniforms
    - Layer transform
    - Layer texture view and sampler
*/

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(perspective) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(0.0);
    out.uv = vec2(0.0, 0.0);
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4(0.0, 0.0, 0.0, 0.0);
}