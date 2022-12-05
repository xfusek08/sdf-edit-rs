
struct PushConstants {
    camera_position: vec4<f32>,
    cot_fov:         f32,
    node_count:      u32,
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

fn phere_view_size(position: vec3<f32>, diameter: f32) -> f32 {
    let distance = length(position - pc.camera_position.xyz);
    return diameter / distance * pc.cot_fov;
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
        
        let position = vertex.xyz;
        let diameter = vertex.w;
        let project_sphere = phere_view_size(position, diameter);
        
        let parent_position = compute_parent_position(node_id, vertex.xyz, vertex.w);
        let parent_diameter = diameter * 2.0;
        
        let projected_parent = phere_view_size(parent_position, parent_diameter);
        
        if (header.has_brick != HEADER_HAS_BRICK_FLAG) {
            return; // brick is empty
        }
        if (projected_parent <=0.0 || project_sphere <= 0.0) {
            return; // not possible
        }
        if (projected_parent < 0.05) {
            return; // parent will be rendered
        }
        if (project_sphere >= 0.05 && header.is_subdivided == HEADER_SUBDIVIDED_FLAG) {
            return; // Node is too big to be rendered and has children
        }
        let index = atomicAdd(&brick_count, 1u);
        brick_index_buffer[index] = node_id;
    }
}
