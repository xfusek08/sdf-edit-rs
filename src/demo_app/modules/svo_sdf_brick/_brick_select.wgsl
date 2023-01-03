
struct PushConstants {
    domain:                   vec4<f32>,
    camera_projection_matrix: mat4x4<f32>,
    camera_focal_length:      f32,
    node_count:               u32,
    level_break_size:         f32,
}
var<push_constant> pc: PushConstants;

struct ShaderInput {
    @builtin(num_workgroups)         num_workgroups:         vec3<u32>,
    @builtin(workgroup_id)           workgroup_id:           vec3<u32>,
    @builtin(workgroup_size)         workgroup_size:         vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(local_invocation_id)    local_invocation_id:    vec3<u32>,
}

// SVO: Node pool Read-only bind group
// -----------------------------------------------------------------------------------

@group(0) @binding(0) var<storage, read> node_count: u32;
@group(0) @binding(1) var<storage, read> node_headers: array<u32>;
@group(0) @binding(2) var<storage, read> node_payload: array<u32>;
@group(0) @binding(3) var<storage, read> node_vertices: array<vec4<f32>>;
@group(0) @binding(4) var<uniform>       node_pool_capacity: u32;


// Oputput brick instance buffer
// -----------------------------------------------------------------------------------
@group(1) @binding(0) var<storage, read_write> brick_index_buffer: array<u32>;
@group(1) @binding(1) var<storage, read_write> brick_count: atomic<u32>;

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

fn distance_adjustment(position: vec3<f32>, size: f32) -> f32 {
    return 1.0;
    // let distance = length(position - pc.camera_projection_matrix[3].xyz);
    // return 1.0 / (distance + pc.camera_focal_length);
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

@compute
@workgroup_size(128, 1, 1)
fn main(in: ShaderInput) {
    let node_id = in.workgroup_id.x * in.workgroup_size.x + in.local_invocation_id.x;
    if (node_id < node_count) {
        let header = deconstruct_node_header(node_headers[node_id]);
        let vertex = node_vertices[node_id];
        
        let me_position = (vertex.xyz * pc.domain.w) + pc.domain.xyz;
        let me_size = vertex.w * pc.domain.w;
        let project_me_size = bounding_cube_screen_size(me_position, me_size);
        let project_me_size = project_me_size * distance_adjustment(me_position, me_size);
        
        let parent_position = compute_parent_position(node_id, me_position, me_size);
        let parent_size = me_size * 2.0;
        let projected_parent_size = bounding_cube_screen_size(parent_position, parent_size);
        let projected_parent_size = projected_parent_size * distance_adjustment(parent_position, parent_size);
        
        let treshhold = 0.1 * pc.level_break_size;
        
        if (header.has_brick != HEADER_HAS_BRICK_FLAG) {
            return; // brick is empty
        }
        if (projected_parent_size <=0.0 || project_me_size <= 0.0) {
            return; // not possible
        }
        if (projected_parent_size < treshhold) {
            return; // parent will be rendered
        }
        if (project_me_size >= (1.01 * treshhold) && header.is_subdivided == HEADER_SUBDIVIDED_FLAG) {
            return; // Node is too big to be rendered and has children
        }
        let index = atomicAdd(&brick_count, 1u);
        brick_index_buffer[index] = node_id;
    }
}
