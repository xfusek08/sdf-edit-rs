
struct ShaderInput {
    @builtin(num_workgroups)         num_workgroups:         vec3<u32>,
    @builtin(workgroup_id)           workgroup_id:           vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(local_invocation_id)    local_invocation_id:    vec3<u32>,
}

// Work assigment uniform data
// -----------------------------------------------------------------------------------

struct SVOWorkAssignment {
    svo_boundding_cube: vec4<f32>, // bounding cube of the SVO in world space (xzy, distance from center to side)
    max_voxel_size: f32,           // minimum voxel size in world space - svo will  be divided until voxel size is smaller than this value
}
@group(0) @binding(0) var<uniform> work_assigment: SVOWorkAssignment;

// SVO: Node pool bind group
// -----------------------------------------------------------------------------------

@group(1) @binding(0) var<storage, read_write> node_count: atomic<u32>; // number of nodes in tiles buffer, use to atomically add new nodes
@group(1) @binding(1) var<storage, read_write> node_headers: array<u32>;
@group(1) @binding(2) var<storage, read_write> node_payload: array<u32>;
@group(1) @binding(3) var<storage, read_write> node_vertices: array<vec4<f32>>;
@group(1) @binding(4) var<uniform>             node_pool_capacity: u32; // maximum number of nodes in tiles buffer

// SVO: Brick pool bind group
// -----------------------------------------------------------------------------------

@group(2) @binding(0) var brick_atlas: texture_storage_3d<r32float, write>;
@group(2) @binding(1) var<storage, read_write> brick_count: atomic<u32>; // number of bricks in brick texture, use to atomically add new bricks
@group(2) @binding(2) var<uniform> brick_pool_side_size: u32; // Number of bricks in one side of the brick atlas texture

// Dispatch Output buffer which is input from previous dispatch and output for next dispatch
// -----------------------------------------------------------------------------------

struct DispatchOutput {
    to_evaluate_nodes: u32,
    start_index:       u32,
}
@group(3) @binding(0) var<storage, read_write> dispatch_output: DispatchOutput;

// Function
// -----------------------------------------------------------------------------------

fn in_voxel(voxel_size: f32, dinstance: f32) -> bool {
    // return true if distance is smaller than voxel size, using square root (might not inclue a corned on voxel cbude)
    // TODO: use max-norm for evaluating this
    return abs(dinstance) < 1.4142136 * voxel_size;
}

fn sample_sdf(position: vec3<f32>) -> f32 {
    // TODO: use max-norm for evaluating this
    
    // tmp - only one sphere
    var sphere_center = vec3<f32>(0.0, 0.0, 0.0);
    var sphere_radius = 0.5;
    return length(position - sphere_center) - sphere_radius;
}

fn brick_index_to_coords(index: u32) -> vec3<u32> {
    var side_size = brick_pool_side_size;
    return  vec3<u32>(
        index % side_size,
        (index / side_size) % side_size,
        (index / side_size) / side_size
    );
}

fn bounding_cube_transform(bc: vec4<f32>, position: vec3<f32>) -> vec3<f32> {
    return bc.w * position + bc.xyz;
}

struct Node {
    index:   u32,
    header:  u32,
    payload: u32,
    vertex:  vec4<f32>,
}
fn load_node(node_index: u32) -> Node {
    var node: Node;
    node.index   = node_index;
    node.header  = node_headers[node_index];
    node.payload = node_payload[node_index];
    node.vertex  = node_vertices[node_index];
    return node;
}

fn write_to_brick(voxel_coords: vec3<u32>, distance: f32) {
    textureStore(brick_atlas, vec3<i32>(voxel_coords), vec4<f32>(distance, 0.0, 0.0, 0.0));
    // TODO: write value to padding voxels
}

let BRICK_IS_EMPTY = 0u;
let BRICK_IS_BOUONDARY = 1u;
let BRICK_IS_FILLED = 2u;
struct BrickEvaluationResult {
    brick_type: u32,
    voxel_size: f32,
}
var<workgroup> divide: atomic<u32>;
var<workgroup> brick_index: u32;
fn evaluate_node_brick(in: ShaderInput, node: Node) -> BrickEvaluationResult {
    var result: BrickEvaluationResult;
    
    let branch_coefficients = vec3<i32>(in.local_invocation_id) - 4; // (0,0,0) - (7,7,7) => (-4,-4,-4) - (3,3,3)
    let voxel_size = 1.0 / 8.0; // hopefully the only FP division and possibly optimize into multiplication by 0.5, 0.25 etc.
    let half_step = voxel_size * 0.5;
    let shift_vector = voxel_size * vec3<f32>(branch_coefficients) + half_step;
    let voxel_center_local = bounding_cube_transform(node.vertex, shift_vector);
    let voxel_center_global = bounding_cube_transform(work_assigment.svo_boundding_cube, voxel_center_local);
    let voxel_size_local = voxel_size * node.vertex.w;
    let voxel_size_global = voxel_size_local * work_assigment.svo_boundding_cube.w;
    let sdf_value = sample_sdf(voxel_center_global);
    
    // vote if voxel intersects sdf surface
    atomicStore(&divide, 0u);
    if (in_voxel(voxel_size, sdf_value)) {
        atomicAdd(&divide, 1u);
    }
    workgroupBarrier(); // synchronize witing of whole group if to divide or not
    
    if (atomicLoad(&divide) != 0u) { // full workgroup branching
        // Save evaluated volume into a new brick
        
        // Take next brick index
        if (in.local_invocation_index == 0u) {
            brick_index = atomicAdd(&brick_count, 1u);
        }
        workgroupBarrier();  // synchronize allocation of brick index
        
        // All threads in group will find voxel coordinate in brick pool based on the brick index
        let brick_coords = brick_index_to_coords(brick_index);
        
        // Get coordinates of voxel in brick (10 = 8 + 2 padding)
        let voxel_coords = 10u * brick_coords + in.local_invocation_id + vec3<u32>(1u, 1u, 1u);
            
        // save voxel value
        write_to_brick(voxel_coords, sdf_value);
        
        // update node payload
        if (in.local_invocation_index == 0u) {
            // encode brick coordinates into payoad integer
            node_payload[node.index] = ((brick_coords.x & 0x3FFu) << 20u) | ((brick_coords.y & 0x3FFu) << 10u) | (brick_coords.z & 0x3FFu);
        }
        
        // return value
        result.brick_type = BRICK_IS_BOUONDARY;
    } else {
        // we suppose that when no boundary crossed any voxel then foolowing condition resolves same for all threads in group
        if (sdf_value < 0.0) {
            result.brick_type = BRICK_IS_FILLED;
        } else {
            result.brick_type = BRICK_IS_EMPTY;
        }
        
        // update node payload
        if (in.local_invocation_index == 0u) {
            node_payload[node.index] = result.brick_type; // TODO: solid color?
        }
    }
    
    result.voxel_size = voxel_size_global;
    workgroupBarrier(); // synchronize writing into node_payload buffer and brick atlas
    return result;
}

// Subdividing node and creating a initialized child tile

let HEADER_NOT_SUBDIVIDED_NO_HEADER = 0u;
let HEADER_FLAGS_MASK = 0xC0000000u;
let HEADER_SUBDIVIDED_FLAG = 0x80000000u;
let HEADER_HAS_BRICK_FLAG = 0x40000000u;
let HEADER_TILE_INDEX_MASK = 0x3FFFFFFFu;
var<workgroup> tile_start_index: u32;
fn subdivide_node(in: ShaderInput, node: Node) {
    
    // 1) allocate new node tile and set reference to it in node header with subdivide flag
    if (in.local_invocation_index == 0u) {
        tile_start_index = atomicAdd(&node_count, 8u);
        var tile_index = (tile_start_index >> 3u) & HEADER_TILE_INDEX_MASK;
        var node_flags = (node_headers[node.index] | HEADER_SUBDIVIDED_FLAG) & HEADER_FLAGS_MASK;
        node_headers[node.index] = node_flags | tile_index;
    }
    workgroupBarrier(); // synch tile_start_index value
    
    // 2) init nodes in tile in 2x2x2 threadsin workgroup
    if (in.local_invocation_id.x < 2u && in.local_invocation_id.y < 2u && in.local_invocation_id.z < 2u) {
        let in_tile_index = tile_start_index + in.local_invocation_id.x + in.local_invocation_id.y * 2u + in.local_invocation_id.z * 4u;
        var child_shifts = vec3<f32>(in.local_invocation_id) - 0.5; // (0,0,0) - (1,1,1) => (-0.5,-0.5,-0.5) - (0.5,0.5,0.5)
        child_shifts = child_shifts * 0.5; // (-0.5,-0.5,-0.5) - (0.5,0.5,0.5) => (-0.25,-0.25,-0.25) - (0.25,0.25,0.25)
        child_shifts = bounding_cube_transform(node.vertex, child_shifts);
        node_vertices[in_tile_index] = vec4(child_shifts, node.vertex.w * 0.5);
    }
    workgroupBarrier(); // synch updateing node_vertices buffer
}

fn process_node(in: ShaderInput, node: Node) {
    let brick_evalutaion_result = evaluate_node_brick(in, node);
    
    // Divide when voxels are bugger then max_voxel_size
    if (brick_evalutaion_result.brick_type == BRICK_IS_BOUONDARY && brick_evalutaion_result.voxel_size > work_assigment.max_voxel_size) {
        subdivide_node(in, node); // As side effect: New initialized tile is added to node pool and ints first node index is store in tile_start_index, node_header is updated to point to new tile index.
    }
}

@compute
@workgroup_size(8, 8, 8)
fn main(in: ShaderInput) {
    let workgroup_index = in.workgroup_id.x + in.workgroup_id.y * in.num_workgroups.x + in.workgroup_id.z * in.num_workgroups.x * in.num_workgroups.y;
    let thread_zero = workgroup_index == 0u && in.local_invocation_index == 0u;
    let start_index = dispatch_output.start_index;
    let to_evaluate_nodes = dispatch_output.to_evaluate_nodes;
    
    if (start_index == 0u) {
        if (workgroup_index == 0u) {
            process_node(in, Node(0u, 0u, 0u, vec4<f32>(0.0, 0.0, 0.0, 1.0)));
        }
    } else if (workgroup_index < to_evaluate_nodes) {
        let node = load_node(start_index + workgroup_index);
        process_node(in, node);
    }
    
    // prepare next dispatch
    if (thread_zero) {
        let new_start_index = start_index + to_evaluate_nodes;
        dispatch_output.start_index = new_start_index;
        dispatch_output.to_evaluate_nodes = atomicLoad(&node_count) - new_start_index;
    }
}
