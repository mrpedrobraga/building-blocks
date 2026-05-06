struct BlockAppearance {
    material: RenderMaterial,
};

struct RenderMaterial {
    atlas_position: vec2<f32>,
    atlas_size: vec2<f32>,
};

struct WorldUniforms {
    view_proj: mat4x4<f32>,
    global_time: f32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
};

struct BlockClusterUniforms {
    transform: mat4x4<f32>,
    size: vec3<u32>,
    _padding: u32
}

/* Universe Bind Group */
@group(0) @binding(0) var<storage, read> block_appearance_palette: array<BlockAppearance>;
@group(0) @binding(1) var material_atlas: texture_2d<f32>;
@group(0) @binding(2) var material_atlas_s: sampler;

/* World Bind Group */
@group(1) @binding(0) var<uniform> world_uniforms: WorldUniforms;

/* Block Group Bind Group */
@group(2) @binding(0) var<uniform> block_group_uniforms: BlockClusterUniforms; 
@group(2) @binding(1) var<storage, read> block_group_data: array<u32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(perspective) uv: vec2<f32>,
    @location(1) block_id: u32,
    @location(2) light_factor: f32,
};

const POSITIONS = array<vec3<f32>, 8>(
    vec3<f32>(0, 0, 0), vec3<f32>(1, 0, 0), vec3<f32>(1, 1, 0), vec3<f32>(0, 1, 0),
    vec3<f32>(0, 0, 1), vec3<f32>(1, 0, 1), vec3<f32>(1, 1, 1), vec3<f32>(0, 1, 1)
);

const INDICES = array<u32, 36>(
    // Front Face (-Z): Outward normal is -Z
    0u, 1u, 2u, 0u, 2u, 3u,
    
    // Back Face (+Z): Outward normal is +Z
    5u, 4u, 7u, 5u, 7u, 6u,
    
    // Left Face (-X): Outward normal is -X
    4u, 0u, 3u, 4u, 3u, 7u,
    
    // Right Face (+X): Outward normal is +X
    1u, 5u, 6u, 1u, 6u, 2u,
    
    // Bottom Face (-Y): Outward normal is -Y
    1u, 0u, 4u, 1u, 4u, 5u,
    
    // Top Face (+Y): Outward normal is +Y
    3u, 2u, 6u, 3u, 6u, 7u
);

const UV_DATA = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0), // 0: Bottom-Left
    vec2<f32>(1.0, 1.0), // 1: Bottom-Right
    vec2<f32>(1.0, 0.0), // 2: Top-Right
    vec2<f32>(0.0, 1.0), // 0: Bottom-Left
    vec2<f32>(1.0, 0.0), // 2: Top-Right
    vec2<f32>(0.0, 0.0)// 3: Top-Left
);

const FACE_LIGHTING = array<f32, 6>(
    0.5, 0.8, 0.8, 0.7, 0.7, 1.0
);

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let raw_block_id = block_group_data[i_idx];
    if raw_block_id == 0u {
        return VertexOutput(vec4<f32>(0.0), vec2<f32>(0.0), 0, 0.0);
    }
    let block_id = raw_block_id - 1;

    let size = block_group_uniforms.size.xyz;
    let x = i_idx % size.x;
    let y = (i_idx / size.x) % size.y;
    let z = i_idx / (size.x * size.y);
    let grid_pos = vec3<f32>(f32(x), f32(y), f32(z));

    let corner_idx = INDICES[v_idx];
    let local_pos = POSITIONS[corner_idx];

    let face_idx = v_idx / 6u;
    let uv_idx = v_idx % 6u; // Which vertex of the face are we on?

    var world_pos = block_group_uniforms.transform * vec4<f32>(grid_pos + local_pos, 1.0);

    var out: VertexOutput;
    out.clip_position = world_uniforms.view_proj * world_pos;
    out.uv = UV_DATA[uv_idx];
    out.block_id = block_id;
    out.light_factor = FACE_LIGHTING[face_idx];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = block_appearance_palette[in.block_id].material;
    let atlas_pixel_size = vec2<f32>(textureDimensions(material_atlas));
    let atlas_uv = mix(material.atlas_position, material.atlas_position + material.atlas_size, in.uv) / atlas_pixel_size;
    let col = textureSample(material_atlas, material_atlas_s, atlas_uv);
    return col * in.light_factor;
}