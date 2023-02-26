
struct PushConstants {
    view_projection:     mat4x4<f32>,
    camera_position:     vec4<f32>,
    domain:              vec4<f32>, // bounding cube
    camera_focal_length: f32,
    brick_scale:         f32,
    brick_atlas_stride:  f32,
    brick_voxel_size:    f32,
    show_flags:          u32,
}
var<push_constant> pc: PushConstants;

let SHOW_SOLID      = 0x01u; // 0b00000001;
let SHOW_NORMALS    = 0x02u; // 0b00000010;
let SHOW_STEP_COUNT = 0x04u; // 0b00000100;
let SHOW_DEPTH      = 0x08u; // 0b00001000;
let JUST_ROOT       = 0x10u; // 0b00010000;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct InstanceInput {
    @location(1) node_index: u32,
    @location(2) instance_id: u32,
}

// SVO: Node pool Read-only bind group
// -----------------------------------------------------------------------------------

@group(0) @binding(0) var<storage, read> node_count: u32;
@group(0) @binding(1) var<storage, read> node_headers: array<u32>;
@group(0) @binding(2) var<storage, read> node_payload: array<u32>;
@group(0) @binding(3) var<storage, read> node_vertices: array<vec4<f32>>;
@group(0) @binding(4) var<uniform>       node_pool_capacity: u32;


// SVO: Brick pool Read-only bind group
// -----------------------------------------------------------------------------------

@group(1) @binding(0) var                brick_atlas:          texture_3d<f32>;
@group(1) @binding(1) var                brick_atlas_sampler:  sampler;
@group(1) @binding(2) var<storage, read> brick_count:          atomic<u32>; // Number of bricks in brick texture, use to atomically add new bricks
@group(1) @binding(3) var<uniform>       brick_pool_side_size: u32;         // Number of bricks in one side of the brick atlas texture


// Instance buffer where currently evaluated svo has one transform mer instance
// -----------------------------------------------------------------------------------
@group(2) @binding(0) var<storage, read> instance_transforms:         array<mat4x4<f32>>;
@group(2) @binding(1) var<storage, read> instance_inverse_transforms: array<mat4x4<f32>>;
@group(2) @binding(2) var<uniform>       instance_count:              u32;

let HEADER_TILE_INDEX_MASK = 0x3FFFFFFFu;
let HEADER_SUBDIVIDED_FLAG = 0x80000000u;
let HEADER_HAS_BRICK_FLAG = 0x40000000u;

struct NodeHeader {
    has_brick: u32,
    is_subdivided: u32,
    tile_index: u32,
}

fn deconstruct_node_header(node_header: u32) -> NodeHeader {
    return NodeHeader(
        node_header & HEADER_HAS_BRICK_FLAG,
        node_header & HEADER_SUBDIVIDED_FLAG,
        node_header & HEADER_TILE_INDEX_MASK,
    );
}

let M4_IDENTITY = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
);

fn translate(translation: vec3<f32>) -> mat4x4<f32> {
    var res = M4_IDENTITY;
    res[3][0] = translation.x;
    res[3][1] = translation.y;
    res[3][2] = translation.z;
    return res;
}

fn scale(scaling: f32) -> mat4x4<f32> {
    var res = M4_IDENTITY;
    res[0][0] = scaling;
    res[1][1] = scaling;
    res[2][2] = scaling;
    return res;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    
    @location(0) frag_pos: vec3<f32>,
    
    @location(1) @interpolate(flat) brick_lookup_shift: vec3<f32>,
    @location(2) @interpolate(flat) brick_local_camera_pos: vec4<f32>,
    
    @location(3) @interpolate(flat) brick_to_local_transform_1: vec4<f32>,
    @location(4) @interpolate(flat) brick_to_local_transform_2: vec4<f32>,
    @location(5) @interpolate(flat) brick_to_local_transform_3: vec4<f32>,
    @location(6) @interpolate(flat) brick_to_local_transform_4: vec4<f32>,
    
    @location(7)  @interpolate(flat) local_to_brick_transform_1: vec4<f32>,
    @location(8)  @interpolate(flat) local_to_brick_transform_2: vec4<f32>,
    @location(9)  @interpolate(flat) local_to_brick_transform_3: vec4<f32>,
    @location(10) @interpolate(flat) local_to_brick_transform_4: vec4<f32>,
    
    // tmp
    @location(11) @interpolate(flat) subdivided: u32,
    // end tmp
};

fn calculate_atlas_lookup_shift(index: u32) -> vec3<f32> {
    let payload = node_payload[index];
    let x = (payload >> 20u) & 0x3FFu;
    let y = (payload >> 10u) & 0x3FFu;
    let z = payload & 0x3FFu;
    let brick_coord = vec3<f32>(f32(x), f32(y), f32(z));
    return (pc.brick_atlas_stride * brick_coord) + vec3<f32>(pc.brick_voxel_size);
}

fn bounding_cube_transform(bc: vec4<f32>, position: vec3<f32>) -> vec3<f32> {
    return bc.w * position + bc.xyz;
}

@vertex
fn vs_main(vertex_input: VertexInput, instance_input: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    
    // values for root node display
    var node_vertex = pc.domain;
    out.brick_lookup_shift = vec3(pc.brick_voxel_size);
    
    // Set values for non-root nodes
    // TODO maybe make a directive in preprocessor and make two versions of the shader
    if ((pc.show_flags & JUST_ROOT) == 0u) {
        node_vertex = node_vertices[instance_input.node_index];
        node_vertex = vec4<f32>(
            (node_vertex.xyz * pc.domain.w) + pc.domain.xyz,
            node_vertex.w * pc.domain.w,
        );
        out.brick_lookup_shift = calculate_atlas_lookup_shift(instance_input.node_index);
    }
    
    let brick_shift = node_vertex.www * 0.5 - node_vertex.xyz;
    let transform = instance_transforms[instance_input.instance_id];
    let inverse_transform = instance_inverse_transforms[instance_input.instance_id];
    
    let brick_to_local_transform = scale(1.0 / node_vertex.w) * translate(brick_shift) * inverse_transform;
    
    // inverse(brick_to_local_transform):
    let local_to_brick_transform = transform * translate(-brick_shift) * scale(node_vertex.w);
    
    let position = transform * vec4(node_vertex.w * vertex_input.position + node_vertex.xyz, 1.0);
    
    out.position = pc.view_projection * position;
    out.frag_pos = position.xyz;
    out.brick_local_camera_pos = (brick_to_local_transform * pc.camera_position);
    
    out.brick_to_local_transform_1 = brick_to_local_transform[0];
    out.brick_to_local_transform_2 = brick_to_local_transform[1];
    out.brick_to_local_transform_3 = brick_to_local_transform[2];
    out.brick_to_local_transform_4 = brick_to_local_transform[3];
    
    out.local_to_brick_transform_1 = local_to_brick_transform[0];
    out.local_to_brick_transform_2 = local_to_brick_transform[1];
    out.local_to_brick_transform_3 = local_to_brick_transform[2];
    out.local_to_brick_transform_4 = local_to_brick_transform[3];
    
    out.subdivided = 0u;
    
    // tmp
    let header_data = deconstruct_node_header(node_headers[instance_input.node_index]);
    if header_data.is_subdivided != 0u {
        out.subdivided = 1u;
    }
    // end tmp
    
    return out;
}

// =================================================================================================
//                                       FRAGMENT SHADER
// =================================================================================================

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    dist: f32,
};

// Founds length of ray until it exits the rendered cube
// Computed in ray-marcher space
// This function was inspired by:
//    https://medium.com/@bromanz/another-view-on-the-classic-ray-aabb-intersection-algorithm-for-bvh-traversal-41125138b525
//    - implemented reduced version of the efficient slab test algorithm
fn get_distance_to_end_of_brick(position: vec3<f32>, direction: vec3<f32>) -> f32 {
    
    // prepare bb of current box
    var maxCorner = vec3<f32>(1.0, 1.0, 1.0);
    var minCorner = vec3<f32>(0.0, 0.0, 0.0);
    
    var inverseRayDir = 1.0 / direction;
    var tMinV0 = (minCorner - position) * inverseRayDir;
    var tMaxV0 = (maxCorner - position) * inverseRayDir;
    
    var tMaxV = max(tMinV0, tMaxV0);
    
    return min(tMaxV.x, min(tMaxV.y, tMaxV.z));
}

fn sample_volume_distance(in: VertexOutput, act_position: vec3<f32>,) -> f32 {
    return textureSample(
        brick_atlas,
        brick_atlas_sampler,
        act_position * pc.brick_scale + in.brick_lookup_shift
    ).r;
}

let NORMAL_OFFSET = 0.05;

/// Compute normal (gradient of sdf) for given point in volume
/// see: https://iquilezles.org/articles/normalsSDF/
fn get_normal(in: VertexOutput, act_position: vec3<f32>, current_distance: f32) -> vec3<f32> {
    let e = vec2<f32>(NORMAL_OFFSET, 0.0);
    let n = vec3<f32>(
        sample_volume_distance(in, act_position + e.xyy),
        sample_volume_distance(in, act_position + e.yxy),
        sample_volume_distance(in, act_position + e.yyx)
    ) - current_distance;
    return normalize(n);
}

// Computing basic Phong lighting
fn get_hit_color(pos: vec3<f32>, normal: vec3<f32>, to_local_matrix: mat4x4<f32>) -> vec4<f32> {
    let lightPos = (to_local_matrix * vec4<f32>(100.0, 100.0, 100.0, 1.0)).xyz;
    let local_camera_pos = (to_local_matrix * pc.camera_position).xyz;
    let lightColor = vec3<f32>(1.0, 1.0, 1.0);
    let ambient = vec3<f32>(1.0, 1.0, 1.0) * 0.25;
    let objectColor = vec3<f32>(0.8, 0.5, 0.3); // TODO: get color from model/voxel ??
    let specularStrength = 0.1; // TODO: get shader details from model
    
    // diffuse
    let lightDir = normalize(lightPos);
    let diff = max(dot(normal, lightDir), 0.0);
    let diffuse = diff * lightColor;
    
    // specular
    let viewDir = normalize(local_camera_pos - pos);
    let reflectDir = reflect(-lightDir, normal);
    let spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    let specular = specularStrength * spec * lightColor;
    
    let result = (ambient + diffuse + specular) * objectColor;
    return vec4<f32>(result, 1.0);
}

let HIT_DISTANCE: f32 = 0.01;
let MAX_STEPS: u32 = 50u;

struct HitResult {
    hit:          bool,
    color:        vec4<f32>,
    position:     vec3<f32>,
    max_distance: f32,
    normal:       vec3<f32>,
    steps:        u32,
}

fn ray_march(in: VertexOutput, origin: vec3<f32>, brick_to_local_transform: mat4x4<f32>) -> HitResult {
    var ray: Ray;
    ray.origin = origin;
    ray.direction = normalize(origin - in.brick_local_camera_pos.xyz);
    ray.dist = 0.0;
    
    var hit = HitResult(
        false,
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        get_distance_to_end_of_brick(ray.origin, ray.direction),
        vec3<f32>(0.0, 0.0, 0.0),
        0u
    );
    
    loop {
        let act_position = ray.dist * ray.direction + ray.origin;
        let dist_to_volume = sample_volume_distance(in, act_position);
        
        if (dist_to_volume < HIT_DISTANCE) {
            hit.hit = true;
            hit.normal = get_normal(in, act_position, dist_to_volume);
            hit.color = get_hit_color(
                act_position,
                hit.normal,
                brick_to_local_transform
            );
            break;
        }
        
        ray.dist += dist_to_volume;
        if ray.dist >= hit.max_distance {
            break;
        }
        
        hit.steps += 1u;
        if (hit.steps > MAX_STEPS) {
            break;
        }
    }
    hit.position = ray.dist * ray.direction + ray.origin;
    return hit;
}
struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let brick_to_local_transform = mat4x4<f32>(
        in.brick_to_local_transform_1,
        in.brick_to_local_transform_2,
        in.brick_to_local_transform_3,
        in.brick_to_local_transform_4,
    );
    
    var out = FragmentOutput(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        in.position.z,
    );
    
    var fragment_pos = (brick_to_local_transform * vec4<f32>(in.frag_pos, 1.0)).xyz;
    
    // Solid brick rendering
    if ((pc.show_flags & SHOW_SOLID) != 0u) {
        var col = vec4<f32>(fragment_pos, 1.0);
        if in.subdivided == 1u {
            col = mix(col, vec4<f32>(1.0, 1.0, 0.0, 1.0), 0.5);
        }
        out.color = col;
        return out;
    }
    
    // Run interior brick raymarching
    let hit = ray_march(in, fragment_pos, brick_to_local_transform);
    
    // calculate color on hit
    if (hit.hit) {
        
        // Depth buffer value correction
        // see: https://stackoverflow.com/questions/53650693/opengl-impostor-sphere-problem-when-calculating-the-depth-value
        let local_to_brick_transform = mat4x4<f32>(
            in.local_to_brick_transform_1,
            in.local_to_brick_transform_2,
            in.local_to_brick_transform_3,
            in.local_to_brick_transform_4,
        );
        let c = pc.view_projection * local_to_brick_transform * vec4(hit.position, 1.0);
        out.depth = c.z / c.w;
        
        // Color calculation
        var color = hit.color;
        if ((pc.show_flags & SHOW_DEPTH) != 0u) {
            let distnace = length(hit.position - fragment_pos);
            color = mix(color, vec4<f32>(1.0, 0.0, 0.0, 1.0), distnace / hit.max_distance);
        }
        if ((pc.show_flags & SHOW_NORMALS) != 0u) {
            color = mix(color, vec4<f32>(hit.normal, 1.0), 0.5);
        }
        if ((pc.show_flags & SHOW_STEP_COUNT) != 0u) {
            color = mix(color, vec4<f32>(0.0, 0.0, 1.0, 1.0), f32(hit.steps) / f32(MAX_STEPS));
        }
        out.color = color;
        return out;
    }
    
    discard;
}
