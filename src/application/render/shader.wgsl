struct BlockDefinition {
    material: RenderMaterial,
};

struct RenderMaterial {
    color: vec3<f32>,
};

struct GlobalUniforms {
    view_proj: mat4x4<f32>,
    cluster_transform: mat4x4<f32>,
    cluster_size: vec3<u32>,
};

@group(0) @binding(0) var<uniform> globals: GlobalUniforms;
@group(0) @binding(1) var<storage, read> palette: array<BlockDefinition>;
@group(0) @binding(2) var<storage, read> cluster_voxels: array<u32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Hardcoded cube offsets for 36 vertices (6 faces * 2 triangles * 3 vertices)
const POSITIONS = array<vec3<f32>, 8>(
    vec3<f32>(0,0,0), vec3<f32>(1,0,0), vec3<f32>(1,1,0), vec3<f32>(0,1,0),
    vec3<f32>(0,0,1), vec3<f32>(1,0,1), vec3<f32>(1,1,1), vec3<f32>(0,1,1)
);

const INDICES = array<u32, 36>(
    0u, 2u, 1u, 0u, 3u, 2u, // Front
    4u, 5u, 6u, 4u, 6u, 7u, // Back
    0u, 1u, 5u, 0u, 5u, 4u, // Bottom
    2u, 3u, 7u, 2u, 7u, 6u, // Top
    0u, 4u, 7u, 0u, 7u, 3u, // Left
    1u, 2u, 6u, 1u, 6u, 5u  // Right
);

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let block_id = cluster_voxels[i_idx];
    
    // If `block_id` is 0, the block is empty and we discard it render-wise by collapsing the vertices;
    if (block_id == 0u) {
        return VertexOutput(vec4<f32>(0.0), vec4<f32>(0.0));
    }

    // Construct the cube model!
    let size = globals.cluster_size.xyz;
    let x = i_idx % size.x;
    let y = (i_idx / size.x) % size.y;
    let z = i_idx / (size.x * size.y);
    let grid_pos = vec3<f32>(f32(x), f32(y), f32(z));

    let corner_idx = INDICES[v_idx];
    let local_pos = POSITIONS[corner_idx];
    
    // Transform: Local -> Cluster -> World -> Camera
    let world_pos = globals.cluster_transform * vec4<f32>(grid_pos + local_pos, 1.0);
    
    var out: VertexOutput;
    out.clip_position = globals.view_proj * world_pos;
    out.color = vec4(palette[block_id].material.color, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}