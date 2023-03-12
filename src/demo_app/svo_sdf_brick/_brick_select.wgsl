
struct PushConstants {
    domain:                   vec4<f32>,
    camera_projection_matrix: mat4x4<f32>,
    camera_focal_length:      f32,
    camera_far:               f32,
    camera_near:              f32,
    node_count:               u32,
    level_break_size:         f32,
}
var<push_constant> pc: PushConstants;

struct ShaderInput {
    @builtin(num_workgroups)         num_workgroups:         vec3<u32>,
    @builtin(workgroup_id)           workgroup_id:           vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(local_invocation_id)    local_invocation_id:    vec3<u32>,
}

// SVO: Node pool Read-only bind group
// -----------------------------------------------------------------------------------

@group(0) @binding(0) var<storage, read> node_count:         u32;
@group(0) @binding(1) var<storage, read> node_headers:       array<u32>;
@group(0) @binding(2) var<storage, read> node_payload:       array<u32>;
@group(0) @binding(3) var<storage, read> node_vertices:      array<vec4<f32>>;
@group(0) @binding(4) var<uniform>       node_pool_capacity: u32;

struct NodeHeader {
    has_brick: u32,
    is_subdivided: u32,
    tile_index: u32,
}

const HEADER_TILE_INDEX_MASK = 0x3FFFFFFFu;
const HEADER_SUBDIVIDED_FLAG = 0x80000000u;
const HEADER_HAS_BRICK_FLAG = 0x40000000u;
const ROOT_ID = 0xFFFFFFFFu;

fn deconstruct_node_header(node_header: u32) -> NodeHeader {
    return NodeHeader(
        node_header & HEADER_HAS_BRICK_FLAG,
        node_header & HEADER_SUBDIVIDED_FLAG,
        node_header & HEADER_TILE_INDEX_MASK,
    );
}

// Oputput brick instance buffer
// -----------------------------------------------------------------------------------

struct BrickInstance {
    brick_id: u32,
    instance_id: u32,
}
@group(1) @binding(0) var<storage, read_write> brick_index_buffer: array<BrickInstance>;
@group(1) @binding(1) var<storage, read_write> brick_count:        atomic<u32>;


// Instance buffer where currently evaluated svo has one transform mer instance
// -----------------------------------------------------------------------------------
@group(2) @binding(0) var<storage, read> instance_transforms:         array<mat4x4<f32>>;
@group(2) @binding(1) var<storage, read> instance_inverse_transforms: array<mat4x4<f32>>;
@group(2) @binding(2) var<uniform>       instance_count:              u32;

fn apply_transform(pos: vec3<f32>, transform: mat4x4<f32>) -> vec3<f32> {
    return (transform * vec4<f32>(pos, 1.0)).xyz;
}

fn extract_scaling(m: mat4x4<f32>) -> f32 {
    let x = length(m[0].xyz);
    let y = length(m[1].xyz);
    let z = length(m[2].xyz);
    return max(x, max(y, z));
}

// -----------------------------------------------------------------------------------

/// Compute the screen size of a bounding cube
/// Based on: https://iquilezles.org/articles/sphereproj/
fn bounding_cube_screen_size(center: vec3<f32>, side_size: f32) -> f32 {
    
    // Boundign sphere radius computed from cube size length
    let radius = 0.866025 * side_size; // sqrt(3)/2 * side_size
    
    // vec3  o = (cam*vec4(sph.xyz,1.0)).xyz;
    let o = (pc.camera_projection_matrix * vec4<f32>(center, 1.0)).xyz;
    
    // float r2 = sph.w*sph.w;
    let r2 = radius * radius;
    
    // float z2 = o.z*o.z;
    let z2 = o.z * o.z;
    
    // float l2 = dot(o,o);
    let l2 = dot(o, o);
    
    // return -3.14159*fl*fl*r2*sqrt(abs((l2-r2)/(r2-z2)))/(r2-z2);
    return -3.14159 * pc.camera_focal_length * pc.camera_focal_length * r2 * sqrt(abs((l2 - r2) / (r2 - z2))) / (r2 - z2);
}

// Compute a parent boundign sphere
fn compute_parent_position(node_id: u32, node_position: vec3<f32>, node_diameter: f32) -> vec3<f32> {
    var shift_vector: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
        vec3<f32>( 0.25,  0.25,  0.25),
        vec3<f32>( 0.25,  0.25, -0.25),
        vec3<f32>( 0.25, -0.25,  0.25),
        vec3<f32>( 0.25, -0.25, -0.25),
        vec3<f32>(-0.25, -0.25,  0.25),
        vec3<f32>(-0.25, -0.25, -0.25),
        vec3<f32>(-0.25,  0.25,  0.25),
        vec3<f32>(-0.25,  0.25, -0.25),
    );
    
    let child_index = node_id & 7u; // modulo 8 (in tile index)
    return node_position + (shift_vector[child_index] * node_diameter);
}

fn in_frustum(position: vec3<f32>, diameter: f32) -> bool {
    let p = (pc.camera_projection_matrix * vec4<f32>(position, 1.0));
    let ndc = p.xyz / p.w;
    let low = -1.0 - diameter * 0.5;
    let high = 1.0 + diameter * 0.5;
    return p.w > 0.0 && ndc.x > low && ndc.x < high && ndc.y > low && ndc.y < high && ndc.z > low && ndc.z < high;
}

const WORKGROUP_SIZE = 128u;
@compute
@workgroup_size(128u, 1, 1)
fn main(in: ShaderInput) {
    let node_id = in.workgroup_id.x * WORKGROUP_SIZE + in.local_invocation_id.x;
    if (node_id >= node_count) {
        return;
    }
    
    let header = deconstruct_node_header(node_headers[node_id]);
    let vertex = node_vertices[node_id];
    let treshhold = 0.1 * pc.level_break_size;
    
    // compute size on screen of current node
    let me_position = (vertex.xyz * pc.domain.w) + pc.domain.xyz;
    let me_size = vertex.w * pc.domain.w;
    let me_position_transformed = (pc.camera_projection_matrix * vec4<f32>(me_position, 1.0)).xyz;
    
    // compute size on screen of parent node
    let parent_position = compute_parent_position(node_id, me_position, me_size);
    let parent_size = me_size * 2.0;
    
    if (header.has_brick != HEADER_HAS_BRICK_FLAG) {
        return; // No brick
    }
    
    for (var i = 0u; i < instance_count; i = i + 1u) {
        let transform = instance_transforms[i];
        let scaling = extract_scaling(transform);
        
        let me_position_transformed = apply_transform(me_position, transform);
        let me_size_scaled = me_size * scaling;
        
        if (!in_frustum(me_position_transformed, me_size_scaled)) {
            continue;
        }
        
        let projected_me_size = bounding_cube_screen_size(me_position_transformed, me_size_scaled);
        
        let parent_position_transformed = apply_transform(parent_position, transform);
        let parent_size_scaled = parent_size * scaling;
        let projected_parent_size = bounding_cube_screen_size(parent_position_transformed, parent_size_scaled);
        
        if (projected_parent_size <=0.0 || projected_me_size <= 0.0) {
            continue; // Not possible
        }
        if (projected_parent_size < treshhold) {
            // only root node is supposed to be rendered, but it is not in the node tree, hence add it manually
            if (node_id == 0u) {
                let index = atomicAdd(&brick_count, 1u);
                brick_index_buffer[index] = BrickInstance(ROOT_ID, i);
            }
            continue; // Parent of this node will be rendered instead
        }
        if (projected_me_size >= (1.01 * treshhold) && header.is_subdivided == HEADER_SUBDIVIDED_FLAG) {
            continue; // Node is too big to be rendered and has children which will be rendered instead
        }
        
        let index = atomicAdd(&brick_count, 1u);
        brick_index_buffer[index] = BrickInstance(node_id, i);
    }
}
