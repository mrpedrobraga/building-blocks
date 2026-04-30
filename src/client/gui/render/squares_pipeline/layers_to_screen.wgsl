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

    let pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),  //  |\    (1)
        vec2<f32>(-1.0, -1.0),  //  | \
        vec2<f32>( 1.0, -1.0),  //  |__\  (2) (3)  (ccw)
        
        vec2<f32>(-1.0,  1.0),  //   \‾‾|  (1)
        vec2<f32>( 1.0, -1.0),  //    \ |
        vec2<f32>( 1.0,  1.0),  //     \|  (2) (3) (ccw)
    );

    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0)
    );

    out.clip_position = vec4<f32>(pos[v_idx], 0.0, 1.0);
    out.uv = uvs[v_idx];

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4(in.uv.x, in.uv.y, 0.0, 0.5);
}