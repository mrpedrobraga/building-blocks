/* Universe Bind Group */
@group(0) @binding(0) var<storage, read> block_appearance_palette: array<BlockAppearance>;
@group(0) @binding(1) var material_atlas: texture_2d<f32>;
@group(0) @binding(2) var material_atlas_s: sampler;

/* World Bind Group */
@group(1) @binding(0) var<uniform> world_uniforms: WorldUniforms;

/* Block Group Bind Group */
@group(2) @binding(0) var<uniform> block_group_uniforms: BlockClusterUniforms; 
@group(2) @binding(1) var<storage, read> block_group_data: array<u32>;

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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) triangle_idx: u32,
    @location(1) @interpolate(perspective) local_position: vec4<f32>,
    @location(2) @interpolate(perspective) world_position: vec4<f32>
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

    // Visualize the triangles.
    //return FragmentOutput(depth, vec4(srgb_to_linear(test_colors[in.triangle_idx % 16]), 1.0));

    let sky_color = vec4(0.9, 0.9, 0.9, 1.0);

    let local_camera_position = block_group_uniforms.inv_transform * vec4<f32>(world_uniforms.camera_world_position.xyz, 1.0);
    
    var _ray_unorm = in.local_position.xyz - local_camera_position.xyz;
    var _ray_origin = dda_aabb_step(in.local_position.xyz, -_ray_unorm);
    let ray_direction = normalize(_ray_unorm); // This is `rd`
    let ray_origin = _ray_origin + ray_direction * 0.001; // This is `ro`... for floating point reasons it's slightly biased!

    let dda_primary = dda_traverse(ray_origin, ray_direction);

    if (!dda_primary.hit) { discard; }

    let appearance = block_appearance_palette[dda_primary.hit_block_type - 1];
    let material = appearance.material;
    let atlas_size = vec2<f32>(textureDimensions(material_atlas));
    var atlas_uv = (material.atlas_position + dda_primary.hit_uv * material.atlas_size) / atlas_size;
    col = textureSample(material_atlas, material_atlas_s, atlas_uv);
    //col = vec4(1.0, 1.0, 1.0, 1.0);
    //col = vec4(srgb_to_linear(test_colors[dda_primary.hit_block_type % 16]), 1.0);

    // *Fancy shadows!*
    let light_direction = normalize(vec3(1.0, 1.0, 2.0));
    var luminosity: f32 = 1.0;
    let shadow_ray_origin = dda_primary.hit_position;
    let dda_shadow = dda_traverse(shadow_ray_origin + light_direction * 0.001, light_direction);
    if (dda_shadow.hit) {
        // If we hit a block, we are in shadow.
        luminosity = 0.0;
    } else {
        luminosity = saturate( dot(dda_primary.hit_normal, light_direction) );
    }    
    col = vec4(col.rgb * (0.25 + 0.75 * luminosity), col.a);

    // Ambient Occlusion (AO)
    let ao_strength = 0.8;
    let black = vec4(0.0, 0.0, 0.0, 1.0);
    let hit_voxel = dda_primary.hit_voxel;
    let containing_voxel = hit_voxel + vec3<i32>(dda_primary.hit_normal);
    let fract_position = dda_primary.hit_position - vec3<f32>(containing_voxel);
    let fract_quad = fract_position * fract_position;

    let neighbours = array<vec3<i32>, 6>(
        vec3( 1,  0,  0), vec3(-1,  0,  0),
        vec3( 0,  1,  0), vec3( 0, -1,  0),
        vec3( 0,  0,  1), vec3( 0,  0, -1)
    );

    // In which directions are there blocks to create ao.
    var occlusion : array<f32, 6>;
    for (var i = 0; i < 6; i++) {
        occlusion[i] = f32(get_voxel(containing_voxel + neighbours[i]) != 0);
    }

    // Computing ao on each axis based on how close the hit point
    // is to either the far or near plane in that axis.
    // We mask it with `occlusion`.
    let ao = array<f32, 3>(
        (1.0 - occlusion[0] * ao_curve(ao_strength * fract_quad.x)) *
            (1.0 - occlusion[1] * ao_curve(ao_strength * (1.0 - fract_quad.x))),
        (1.0 - occlusion[2] * ao_curve(ao_strength * fract_quad.y)) *
            (1.0 - occlusion[3] * ao_curve(ao_strength * (1.0 - fract_quad.y))),
        (1.0 - occlusion[4] * ao_curve(ao_strength * fract_quad.z)) *
            (1.0 - occlusion[5] * ao_curve(ao_strength * (1.0 - fract_quad.z)))
    );

    // Combine the ao, skipping the axis that corresponds to the plane we hit.
    // After all, we are always close to the plane we hit, and we dont want all blocks to be black.
    var ao_merged = 1.0;
    for (var i: u32 = 0; i < 3; i++) {
        ao_merged *= select(ao[i], 1.0, i == dda_primary.hit_plane);
    }

    col = mix(black, col, ao_merged);

    // Fancy reflected blocks...
    // TODO: Extract traversal+texturing+lighting into another function so it can be called recursively/iteratively.
    
    // if (dda_primary.hit_block_type > 0) {
    //     let reflected_ray_direction = reflect(ray_direction, dda_primary.hit_normal);
    //     let reflected_ray_origin = dda_primary.hit_position;
    //     let dda_reflection = dda_traverse(reflected_ray_origin + reflected_ray_direction * 0.001, reflected_ray_direction);
    
    //     var col_reflected = sky_color;

    //     if (dda_reflection.hit) {
    //         let r_appearance = block_appearance_palette[dda_reflection.hit_block_type - 1];
    //         let r_material = r_appearance.material;
    //         var r_atlas_uv = (r_material.atlas_position + dda_reflection.hit_uv * r_material.atlas_size) / atlas_size;
        
    //         col_reflected = textureSample(material_atlas, material_atlas_s, r_atlas_uv);

    //         var r_luminosity: f32;
    //         let r_dda_shadow = dda_traverse(dda_reflection.hit_position + light_direction * 0.001, light_direction);
    //         if (r_dda_shadow.hit) {
    //             // If we hit a block, we are in shadow.
    //             r_luminosity = 0.0;
    //         } else {
    //             r_luminosity = saturate( dot(dda_reflection.hit_normal, light_direction) );
    //         }    
    //         col_reflected *= 0.25 + 0.75 * r_luminosity;
    //     }
    //     col = mix(col, col_reflected, 0.1);
    // }

    /* Depth Calculation */    
    let ray_hit_world_pos = block_group_uniforms.transform * vec4(dda_primary.hit_position, 1.0);
    let ray_hit_clip_pos = world_uniforms.view_matrix * ray_hit_world_pos;
    depth = ray_hit_clip_pos.z / ray_hit_clip_pos.w;

    // Simple fog :-)
    let distance = distance(local_camera_position.xyz, ray_origin) + dda_primary.hit_distance;
    col = mix(col, sky_color, saturate(distance * distance * 0.00001));

    return FragmentOutput(depth, col);
}

struct TraversalOutput {
    hit: bool,
    hit_plane: u32,
    hit_distance: f32,
    hit_position: vec3<f32>,
    hit_voxel: vec3<i32>,
    hit_normal: vec3<f32>,
    hit_uv: vec2<f32>,
    hit_block_type: u32,
}

/// Performs Digital Differential Analysis traversal until something is hit
fn dda_traverse(start: vec3<f32>, direction: vec3<f32>) -> TraversalOutput {
    /*
        Digital Differential Analyzer
        - [More information](https://www.youtube.com/watch?v=NbSee-XM7WA);
    */

    // While performing DDA, we'll move along the grid
    // in consistent integer steps.
    // 
    // TODO: Allow traversing the grid at different cell sizes,
    // as a preparation for LOD traversal :-)
    let voxel_step = vec3<i32>(sign(direction));

    // This is `(rdr/rdx, rdr/rdy, rdr/rdz)`.
    // Another way of thinking about `rdr/rdx` is the distance the ray
    // will travel across its length to traverse 1 unit in the x axis,
    // and likewise for the other axes.
    let dda_step = 1.0 / abs(direction);

    // These are the three ray distances from the starting point.
    // As we step through the lattice, we compare which distance is
    // the shortest so we can take the highest leaps we can,
    // without missing any intersections with the lattice.
    var dda_distances: vec3<f32> = dda_step * select(
        start - floor(start),
        floor(start) + 1.0 - start,
        direction > vec3(0.0)
    );

    var current_voxel: vec3<i32> = vec3<i32>(start);
    var current_voxel_block_type: u32;
    var last_intersection_plane: u32 = 0; // 0 = YZ; 1 = XZ; 2 = XY;

    var dist_to_volume_min = abs(start);
    var dist_to_volume_max = abs(vec3<f32>(block_group_uniforms.size) - start);
    let min_dists = min(dist_to_volume_min, dist_to_volume_max);

    // TODO: Maybe I can use this to get the position on the front face, too?
    if (min_dists.x <= min_dists.y && min_dists.x <= min_dists.z) {
        last_intersection_plane = 0;
    } else if (min_dists.y <= min_dists.x && min_dists.y <= min_dists.z) {
        last_intersection_plane = 1;
    } else {
        last_intersection_plane = 2;
    }

    var raymarching_iteration_idx = 0;
    let max_step_count = i32(block_group_uniforms.size.x + block_group_uniforms.size.y + block_group_uniforms.size.z);
    for(; raymarching_iteration_idx < max_step_count; raymarching_iteration_idx++) {
        if(!aabb_contains(current_voxel, vec3(0), vec3<i32>(block_group_uniforms.size))) {
            return TraversalOutput(false, last_intersection_plane, 0.0, vec3(0.0), vec3(0), vec3(0.0), vec2(0.0), 0);
        }

        current_voxel_block_type = get_voxel(current_voxel);

        // TODO: Use a more sophisticated way of detecting a hit.
        if (current_voxel_block_type != 0 /* 0 = AIR */) {
            break;
        }

        if (dda_distances.x < dda_distances.y) {
            if (dda_distances.x < dda_distances.z) {
                dda_distances.x += dda_step.x; // move one step in the x axis
                current_voxel.x += voxel_step.x; // we always go to an adjacent voxel :-)
                last_intersection_plane = 0;   // x axis = yz plane
            } else {
                dda_distances.z += dda_step.z;
                current_voxel.z += voxel_step.z;
                last_intersection_plane = 2;
            }
        } else {
            if (dda_distances.y < dda_distances.z) {
                dda_distances.y += dda_step.y;
                current_voxel.y += voxel_step.y;
                last_intersection_plane = 1;
            } else {
                dda_distances.z += dda_step.z;
                current_voxel.z += voxel_step.z;
                last_intersection_plane = 2;
            }
        }
    }

    /* Ray has hit something */
    var travelled_distance: f32;

    if raymarching_iteration_idx == 0 {
        travelled_distance = 0.0;
    } else if (last_intersection_plane == 0) {
        travelled_distance = dda_distances.x - dda_step.x;
    } else if (last_intersection_plane == 1) {
        travelled_distance = dda_distances.y - dda_step.y;
    } else /*if (last_intersection_plane == 2)*/ {
        travelled_distance = dda_distances.z - dda_step.z;
    }

    let ray_hit_position = start + direction * travelled_distance;
    let ray_hit_uvw = ray_hit_position - floor(ray_hit_position);

    var uv: vec2<f32>;
    var normal: vec3<f32>;

    if (last_intersection_plane == 0) {
        uv = ray_hit_uvw.zy;
        normal.x = f32( -voxel_step.x );
    } else if (last_intersection_plane == 1) {
        uv = ray_hit_uvw.xz;
        normal.y = f32( -voxel_step.y );
    } else /*if (last_intersection_plane == 2)*/ {
        uv = ray_hit_uvw.xy;
        normal.z = f32( -voxel_step.z );
    }

    var result = TraversalOutput();
    result.hit = true;
    result.hit_distance = travelled_distance;
    result.hit_position = ray_hit_position;
    result.hit_normal = normal;
    result.hit_uv = uv;
    result.hit_block_type = current_voxel_block_type;
    result.hit_plane = last_intersection_plane;
    result.hit_voxel = current_voxel;
    return result;
}


/// Performs a single step of Digital Differential Analysis to go from a point in a back face plane
/// to the front face.
fn dda_aabb_step(backface_point: vec3<f32>, ray_to_camera: vec3<f32>) -> vec3<f32> {
    let start = backface_point;
    let camera_dist = length(ray_to_camera);
    let direction = normalize(ray_to_camera);
    let dda_step = 1.0/abs(direction);
    let dda_distances = dda_step * select(
        start,
        vec3<f32>(block_group_uniforms.size) - start,
        direction > vec3(0.0)
    );
    var travelled_distance = min(camera_dist,min(min(dda_distances.x, dda_distances.y), dda_distances.z));
    let end = start + direction * (travelled_distance - 1e-4);
    return end;
}

/// From a voxel position, retrieves an index to sample from the voxel buffer.
fn voxel_position_encode(local_position: vec3<u32>) -> u32 {
    return local_position.x +
        local_position.y * block_group_uniforms.size.x +
        local_position.z * block_group_uniforms.size.x * block_group_uniforms.size.y;
}

fn get_voxel(local_position: vec3<i32>) -> u32 {
    let current_voxel_idx_in_buffer = voxel_position_encode(vec3<u32>(local_position));
    if current_voxel_idx_in_buffer > arrayLength(&block_group_data) { return 0; };
    return block_group_data[current_voxel_idx_in_buffer];
}

/// Returns whether a point is inside an AABB (end-exclusive).
fn aabb_contains(
    point: vec3<i32>,
    aabb_min: vec3<i32>,
    aabb_max: vec3<i32>
) -> bool {
    return all(point >= aabb_min) && all(point < aabb_max);
}

fn aabb_contains_f(
    point: vec3<f32>,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>
) -> bool {
    return all(point >= aabb_min) && all(point < aabb_max);
}

/// A simple procedurally generated 'missing texture' in case any voxels
/// refer to a texture that's not in the texture buffer.
fn tex_missing_texture(uv: vec2<f32>) -> vec4<f32> {
    let grid = floor(uv * vec2(2.0));
    if (i32(grid.x + grid.y) % 2 == 0) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4(1.0, 0.0, 1.0, 1.0);
    }
}

fn ao_curve(x: f32) -> f32 {
    return 0.5*x*x;
}

/// Converts a colour from SRGB to linear.
/// Because of automatic Linear to SRGB conversion the engine uses,
/// when trying to render accurate colours for debugging (like UV, normals, etc),
/// we use this to "cancel out" the conversion.
fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = 0.04045;
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = srgb / vec3<f32>(12.92);
    
    // Select higher or lower based on component-wise comparison
    return select(lower, higher, srgb > vec3<f32>(cutoff));
}