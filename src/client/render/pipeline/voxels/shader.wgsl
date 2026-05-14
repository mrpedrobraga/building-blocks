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
    let local_position = vec4(vertex_pos * vec3<f32>(block_group_uniforms.size), 1.0);
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

    /* Digital Differential Analyzer */
    
    /* 
                    |
                    X  A ray originating from inside a block will intersect the lattice either on the x or y axis,
                  . |  depending on whether whether its `ray_direction`'s x coordinate or y is greater.
    -------------Y---  
    |    | rd .     |  (mx, my) = (dda_sides_dist)
    | my | .        |  ro = ray_origin
    |   ro----------|  rd = ray_direction
    |         mx    |  Y and X are possible intersection points (in the example illustrated we'd pick Y since my < mx).
    |               |
    |               |  Y = my * |d/dy|    (Since `d = 1.0` this is `1.0/dy`. Absolute value to prevent sign flippage.)
    -----------------  X = mx * |d/dx|    These are precomputed on `dda_step`.

    ...X.       |        |        |        |
       | .....  |        |        |        |              If we consider a ray travel in only the X axis,
       |      ..X..      |        |        |              we see that it always steps of the same size (x = 1.0; y = )
       |        |  ..... |        |        |
       |        |       .X...     |        |
       |        |        |   .....|        |
       |        |        |        X....    |
       |        |        |        |    ....X
       |        |        |        |        |.....
       |        |        |        |        |

                                          It's wonderful that as the photon traverses the grid,
                                          we make decisions on which direction it steps in,
                                          but for each direction, the step size is ALWAYS the same (`d/d_axis`)
                                          and so we can precompute it.
    
        - [More information](https://www.youtube.com/watch?v=NbSee-XM7WA);

    */

    let world_ray_dir = (in.world_position.xyz - world_uniforms.camera_world_position.xyz);
    let ray_direction = normalize((block_group_uniforms.inv_transform * vec4<f32>(world_ray_dir, 0.0)).xyz); // This is `rd`
    let ray_origin = in.local_position.xyz; // This is `ro`

    // We can precompute "signs" for the direction of changes to the 
    // photon position since these never change
    // (if a ray's, say, X coordinate is increasing, it'll continue to increase, and so on.)
    var dda_voxel_step = vec3<i32>(sign(ray_direction));
    let dda_step = 1.0 / abs(ray_direction); // This is `(d/dx, d/dy, d/dz)`.

    var dda_sides_dist: vec3<f32> = dda_step * select(  // This is `m`
        ray_origin - floor(ray_origin),
        floor(ray_origin) + 1.0 - ray_origin, 
        ray_direction > vec3(0.0)              
    );
    var ray_voxel_position: vec3<i32> = vec3<i32>(ray_origin + ray_direction * 1e-4);
    var ray_last_intersection_plane = 0; // 0 = YZ; 1 = XZ; 2 = XY;
    var current_voxel_block_type: u32;
    
    var raymarching_iteration_idx = 0;
    let max_step_count = i32(max(max(block_group_uniforms.size.x, block_group_uniforms.size.y), block_group_uniforms.size.z));
    for (; raymarching_iteration_idx < max_step_count; raymarching_iteration_idx++) {
        /* If we leave the AABB, give up! */
        if (!is_inside_box(ray_voxel_position, vec3(0, 0, 0), vec3<i32>(block_group_uniforms.size))) {
            discard;
        }
        
        current_voxel_block_type = block_group_data[voxel_position_to_idx(vec3<u32>(ray_voxel_position))];

        /* Stop traversing if the current block is not air. */
        if (current_voxel_block_type != 0) {
            // TODO: Some blocks might have parts of it that aren't solid!
            break;
        }

        // Here is where we pick the intersection point (Y, X or Z)...
        if (dda_sides_dist.x < dda_sides_dist.y) {
            if (dda_sides_dist.x < dda_sides_dist.z) {
                // X (YZ plane)
                dda_sides_dist.x += dda_step.x;           //
                ray_voxel_position.x += dda_voxel_step.x; // we _always_ go to an adjacent voxel when we take a step!
                ray_last_intersection_plane = 0;
            } else {
                // Z (XY plane)
                dda_sides_dist.z += dda_step.z;
                ray_voxel_position.z += dda_voxel_step.z;
                ray_last_intersection_plane = 2;
            }
        } else {
            if (dda_sides_dist.y < dda_sides_dist.z) {
                // Y (XZ plane)
                dda_sides_dist.y += dda_step.y;
                ray_voxel_position.y += dda_voxel_step.y;
                ray_last_intersection_plane = 1;
            } else {
                // Z (XY plane)
                dda_sides_dist.z += dda_step.z;
                ray_voxel_position.z += dda_voxel_step.z;
                ray_last_intersection_plane = 2;
            }
        }
    }

    /* Ray has hit something, for real! */

    var t: f32;
    if (ray_last_intersection_plane == 0) {
        t = dda_sides_dist.x - dda_step.x;
    }
    else if (ray_last_intersection_plane == 1) {
        t = dda_sides_dist.y - dda_step.y;
    }
    else {
        t = dda_sides_dist.z - dda_step.z;
    }

    let ray_hit_pos = ray_origin + ray_direction * t;
    let ray_hit_pos_within_block = ray_hit_pos - floor(ray_hit_pos);

    /* Texturing the hit block (only applies in full LOD) */

    var uv: vec2<f32>;
    var normal: vec3<f32>;

    if (ray_last_intersection_plane == 0) {
        uv = ray_hit_pos_within_block.zy;
        normal.x = -f32(dda_voxel_step.x);
    }
    else if (ray_last_intersection_plane == 1) {
        uv = ray_hit_pos_within_block.xz;
        normal.y = -f32(dda_voxel_step.y);
    }
    else {
        uv = ray_hit_pos_within_block.xy;
        normal.z = -f32(dda_voxel_step.z);
    }

    let appearance = block_appearance_palette[current_voxel_block_type - 1];
    let material = appearance.material;
    let atlas_size = vec2<f32>(textureDimensions(material_atlas));
    let atlas_uv = (material.atlas_position + uv * material.atlas_size) / atlas_size;

    //col = textureSample(material_atlas, material_atlas_s, atlas_uv);
    col = vec4(test_colors[(current_voxel_block_type-1) % 16], 1.0);

    /* Simple Lighting */
    let light = normalize(vec3(0.5, 0.0, 1.0));
    col *= 0.25 + 0.75 * saturate( dot(normal, light) );
    
    let ray_hit_world_pos = block_group_uniforms.transform * vec4(ray_hit_pos, 1.0);
    let ray_hit_clip_pos = world_uniforms.view_matrix * ray_hit_world_pos;
    //depth = t;
    depth = ray_hit_clip_pos.z / ray_hit_clip_pos.w;
    
    //col = vec4(vec3(1.0) - vec3(ray_hit_world_pos.z / 20.0), col.a);
    
    // /* HEATMAP */
    // col = vec4(vec3( f32(raymarching_iteration_idx) / 200.0 ), 1.0);
    // /* Undoes LINEAR to SRGB conversion, useful for visualizing mathematical data. */
    // col = vec4(srgbToLinear(col.rgb), col.a);

    return FragmentOutput(depth, col);
}

fn voxel_position_to_idx(local_position_i: vec3<u32>) -> u32 {
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

fn srgbToLinear(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = 0.04045;
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = srgb / vec3<f32>(12.92);
    
    // Select higher or lower based on component-wise comparison
    return select(lower, higher, srgb > vec3<f32>(cutoff));
}
