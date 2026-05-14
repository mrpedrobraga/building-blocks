struct BlockAppearance {
    material: RenderMaterial,
};

struct RenderMaterial {
    atlas_position: vec2<f32>,
    atlas_size: vec2<f32>,
};

struct WorldUniforms {
    view_matrix: mat4x4<f32>,
    camera_world_position: vec4<f32>,
    global_time: f32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
};

struct BlockClusterUniforms {
    transform: mat4x4<f32>,
    inv_transform: mat4x4<f32>,
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
    @location(1) triangle_idx: u32,
    @location(2) @interpolate(perspective) local_position: vec4<f32>,
    @location(3) @interpolate(perspective) world_position: vec4<f32>
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

const FACE_NORMALS = array<vec3<f32>, 6>(
    vec3<f32>(0.0, 0.0, -1.0),
    vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(-1.0, 0.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, -1.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0)
);

const test_colors: array<vec3<f32>, 16> = array<vec3<f32>, 16>(
    vec3<f32>(1.0, 0.0, 0.0),    // 01: Pure Red
    vec3<f32>(0.0, 1.0, 0.0),    // 02: Pure Green
    vec3<f32>(0.0, 0.0, 1.0),    // 03: Pure Blue
    vec3<f32>(1.0, 1.0, 0.0),    // 04: Bright Yellow
    vec3<f32>(0.0, 1.0, 1.0),    // 05: Electric Cyan
    vec3<f32>(1.0, 0.0, 1.0),    // 06: Vivid Magenta
    vec3<f32>(1.0, 0.5, 0.0),    // 07: Bright Orange
    vec3<f32>(0.5, 0.0, 1.0),    // 08: Deep Purple
    vec3<f32>(0.0, 1.0, 0.5),    // 09: Mint Green
    vec3<f32>(1.0, 0.0, 0.5),    // 10: Hot Pink
    vec3<f32>(0.0, 0.5, 1.0),    // 11: Sky Blue
    vec3<f32>(0.5, 1.0, 0.0),    // 12: Lime Green
    vec3<f32>(0.2, 0.2, 0.2),    // 13: Dark Charcoal
    vec3<f32>(0.8, 0.8, 0.8),    // 14: Light Grey
    vec3<f32>(0.6, 0.4, 0.2),    // 15: Earth Brown
    vec3<f32>(1.0, 1.0, 1.0)     // 16: Pure White
);

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let face_idx = v_idx / 6u;
    let uv_idx = v_idx % 6u; // Which vertex of the face are we on?
    
    let corner_idx = INDICES[v_idx];
    let vertex_pos = POSITIONS[corner_idx];
    let local_position = vec4(vertex_pos * vec3<f32>(block_group_uniforms.size.xyz), 1.0);
    let world_position = block_group_uniforms.transform * local_position;

    var out: VertexOutput;
    out.clip_position = world_uniforms.view_matrix * world_position;
    out.triangle_idx = v_idx + i_idx;
    out.uv = UV_DATA[v_idx % 6];
    out.local_position = local_position;
    out.world_position = world_position;
    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var depth: f32 = in.clip_position.z;
    var col: vec4<f32>;

    let world_ray_dir = (in.world_position.xyz - world_uniforms.camera_world_position.xyz);
    let ray_direction = normalize((block_group_uniforms.inv_transform * vec4<f32>(world_ray_dir, 0.0)).xyz);
    let ray_origin = in.local_position.xyz;

    var photon_voxel_position: vec3<i32> = vec3<i32>(ray_origin + 0.001 * ray_direction);
    
    let inv_dir = abs(1.0 / ray_direction);
    var dda_ray_step = vec3<i32>(sign(ray_direction));
    var dda_sides_dist: vec3<f32> = inv_dir * select(
        ray_origin - floor(ray_origin), 
        floor(ray_origin) + 1.0 - ray_origin, 
        ray_direction > vec3(0.0)
    );

    // if (ray_direction.x < 0.0) {
    //     dda_sides_dist.x = (ray_origin.x - f32(photon_voxel_position.x)) * inv_dir.x;
    // } else {
    //     dda_sides_dist.x = (1.0 - ray_origin.x + f32(photon_voxel_position.x)) * inv_dir.x;
    // }
    // if (ray_direction.y < 0.0) {
    //     dda_sides_dist.y = (ray_origin.y - f32(photon_voxel_position.y)) * inv_dir.y;
    // } else {
    //     dda_sides_dist.y = (1.0 - ray_origin.y + f32(photon_voxel_position.y)) * inv_dir.y;
    // }
    // if (ray_direction.z < 0.0) {
    //     dda_sides_dist.z = (ray_origin.z - f32(photon_voxel_position.z)) * inv_dir.z;
    // } else {
    //     dda_sides_dist.z = (1.0 - ray_origin.z + f32(photon_voxel_position.z)) * inv_dir.z;
    // }

    var ray_hit_face = 0; // 0 = YZ; 1 = XZ; 2 = XY;
    var current_voxel_block_type: u32;

    for (var i = 0; i < 200; i++) {
        if (!is_inside_box(photon_voxel_position, vec3(0, 0, 0), vec3<i32>(block_group_uniforms.size))) {
            discard;
        }
        
        let block_idx = block_index_for_position(vec3<u32>(photon_voxel_position));
        current_voxel_block_type = block_group_data[block_idx];

        /* If the block isn't air... */
        if (current_voxel_block_type != 0) {
            col = vec4(0.0, 0.0, 0.0, 1.0);
            break;
        }

        if (dda_sides_dist.x < dda_sides_dist.y) {
            if (dda_sides_dist.x < dda_sides_dist.z) {
                dda_sides_dist.x += inv_dir.x;
                photon_voxel_position.x += dda_ray_step.x;
                ray_hit_face = 0;
            } else {
                dda_sides_dist.z += inv_dir.z;
                photon_voxel_position.z += dda_ray_step.z;
                ray_hit_face = 2;
            }
        } else {
            if (dda_sides_dist.y < dda_sides_dist.z) {
                dda_sides_dist.y += inv_dir.y;
                photon_voxel_position.y += dda_ray_step.y;
                ray_hit_face = 1;
            } else {
                dda_sides_dist.z += inv_dir.z;
                photon_voxel_position.z += dda_ray_step.z;
                ray_hit_face = 2;
            }
        }
    }

    var t: f32;
    if (ray_hit_face == 0) {
        t = dda_sides_dist.x - inv_dir.x;
    }
    else if (ray_hit_face == 1) {
        t = dda_sides_dist.y - inv_dir.y;
    }
    else {t = dda_sides_dist.z - inv_dir.z;
    }

    let ray_hit_pos = ray_origin + ray_direction * t;
    let ray_hit_pos_within_block = ray_hit_pos - floor(ray_hit_pos);

    var uv: vec2<f32>;
    var normal: vec3<f32>;

    if (ray_hit_face == 0) {
        uv = ray_hit_pos_within_block.zy;
        normal.x = -f32(dda_ray_step.x);
    }
    else if (ray_hit_face == 1) {
        uv = ray_hit_pos_within_block.xz;
        normal.y = -f32(dda_ray_step.y);
    }
    else {
        uv = ray_hit_pos_within_block.xy;
        normal.z = -f32(dda_ray_step.z);
    }

    let appearance = block_appearance_palette[current_voxel_block_type - 1];
    let material = appearance.material;
    let atlas_size = vec2<f32>(textureDimensions(material_atlas));
    let atlas_uv = (material.atlas_position + uv * material.atlas_size) / atlas_size;

    col = textureSample(material_atlas, material_atlas_s, atlas_uv);

    /* Simple Lighting */
    let light = normalize(vec3(0.5, 0.0, 1.0));
    col *= 0.5 + 0.5 * saturate( dot(normal, light) );

    /* Undoes LINEAR to SRGB conversion, useful for visualizing mathematical data. */
    // col = vec4(srgbToLinear(col.rgb), col.a);

    return FragmentOutput(depth, col);
}

fn block_index_for_position(local_position_i: vec3<u32>) -> u32 {
    return local_position_i.x +
        local_position_i.y * block_group_uniforms.size.x +
        local_position_i.z * block_group_uniforms.size.x * block_group_uniforms.size.y;
}

fn is_inside_box(
    point: vec3<i32>,
    box_min: vec3<i32>,
    box_max: vec3<i32>
) -> bool {
    return all(point >= box_min) && all(point < box_max);
}


fn missingTexture(uv: vec2<f32>) -> vec4<f32> {
    let grid = floor(uv * vec2(2.0));
    if (i32(grid.x + grid.y) % 2 == 0) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4(1.0, 0.0, 1.0, 1.0);
    }
}

fn checkerboard(position: vec3<f32>) -> f32 {
    return f32(i32(position.x + position.y + position.z) % 2);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, saturate(p - K.xxx), c.y);
}

fn srgbToLinear(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = 0.04045;
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = srgb / vec3<f32>(12.92);
    
    // Select higher or lower based on component-wise comparison
    return select(lower, higher, srgb > vec3<f32>(cutoff));
}
