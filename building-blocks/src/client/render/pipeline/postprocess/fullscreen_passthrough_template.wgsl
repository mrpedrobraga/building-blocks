/* 
    PASSTHROUGH TEMPLATE

    This renders a fullscreen quad when called with `render_pass.drawn(0..6, 0..1)`
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
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0)
    );

    // 3. Pull the vertex data using the index
    out.clip_position = vec4<f32>(pos[v_idx], 0.0, 1.0);
    out.uv = uvs[v_idx];

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4(in.uv.x, in.uv.y, 0.0, 1.0);
}