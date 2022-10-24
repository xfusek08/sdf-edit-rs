
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
    voxel_size: f32,               // minimum voxel size in world space - divide node if its voxels are bigger then this value
    is_root: u32,                  // is this the root node? [0/1]
    start_index: u32,              // node index from which to start the evaluation
}
@group(0) @binding(0) var<uniform> work_assigment: SVOWorkAssignment;

// SVO: Node pool bind group and associated node - fuinctions
// -----------------------------------------------------------------------------------

@group(1) @binding(0) var<storage, read_write> node_count: atomic<u32>; // number of nodes in tiles buffer, use to atomically add new nodes
@group(1) @binding(1) var<storage, read_write> node_headers: array<u32>;
@group(1) @binding(2) var<storage, read_write> node_payload: array<u32>;
@group(1) @binding(3) var<storage, read_write> node_vertices: array<vec4<f32>>;
@group(1) @binding(4) var<uniform>             node_pool_capacity: u32; // maximum number of nodes in tiles buffer

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

let HEADER_IS_SUBDIVIDED_SHIFT = 31u;
let HEADER_HAS_BRICK_SHIFT = 30u;
let HEADER_TILE_INDEX_MASK = 0x3FFFFFFFu;

/// Combines tile index and flags into single node header integer
/// !!! `is_subdivided` must have value 0 or 1
/// !!! `is_leaf` must have value 0 or 1
fn create_node_header(value: u32, is_subdivided: u32, has_brick: u32) -> u32 {
    return (value & HEADER_TILE_INDEX_MASK) | (is_subdivided << HEADER_IS_SUBDIVIDED_SHIFT) | (has_brick << HEADER_HAS_BRICK_SHIFT);
}

/// Encodes brick location into single integer
fn create_node_brick_payload(brick_location: vec3<u32>) -> u32 {
    return ((brick_location.x & 0x3FFu) << 20u) | ((brick_location.y & 0x3FFu) << 10u) | (brick_location.z & 0x3FFu);
}

// SVO: Brick pool bind group and associated functions
// -----------------------------------------------------------------------------------

@group(2) @binding(0) var brick_atlas: texture_storage_3d<r32float, write>;
@group(2) @binding(1) var<storage, read_write> brick_count: atomic<u32>; // number of bricks in brick texture, use to atomically add new bricks
@group(2) @binding(2) var<uniform> brick_pool_side_size: u32;            // Number of bricks in one side of the brick atlas texture

/// Converts brick index to brick location in brick atlas texture
fn brick_index_to_coords(index: u32) -> vec3<u32> {
    var side_size = brick_pool_side_size;
    return  vec3<u32>(
        index % side_size,
        (index / side_size) % side_size,
        (index / side_size) / side_size
    );
}

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

fn bounding_cube_transform(bc: vec4<f32>, position: vec3<f32>) -> vec3<f32> {
    return bc.w * position + bc.xyz;
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
    brick_location: vec3<u32>,
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
        
        // return value
        result.brick_type = BRICK_IS_BOUONDARY;
        result.brick_location = brick_coords;
    } else {
        // we suppose that when no boundary crossed any voxel then foolowing condition resolves same for all threads in group
        if (sdf_value < 0.0) {
            result.brick_type = BRICK_IS_FILLED;
        } else {
            result.brick_type = BRICK_IS_EMPTY;
        }
    }
    
    result.voxel_size = voxel_size_global;
    return result;
}

/// Allocates a new tile and returns its index
var<workgroup> tile_index: u32;
fn create_tile(in: ShaderInput) -> u32 {
    if (in.local_invocation_index == 0u) {
        if (atomicLoad(&node_count) < node_pool_capacity - 8u) {
            // tile might still exceed node pool capacity
            let first_tile_node_index = atomicAdd(&node_count, 8u);
            if (node_pool_capacity > (first_tile_node_index + 8u)) {
                tile_index = first_tile_node_index >> 3u;
            } else {
                // Refuse to initialize the tile becauase there is no more capacity node count increment has to be corrected.
                tile_index = 0u;
                atomicSub(&node_count, 8u);
            }
        }
    }
    workgroupBarrier(); // synch tile_start_index value
    return tile_index;
}

/// Initializes a new tile by computing vertices for each node and writing them into node_vertices buffer
fn initialize_tile(in: ShaderInput, parent_node: Node, tile_index: u32) {
    
    // Enters 2x2x2 subgroup of threads
    if (in.local_invocation_id.x < 2u && in.local_invocation_id.y < 2u && in.local_invocation_id.z < 2u) {
        let start_node_tile = tile_index << 3u;
        let node_index = start_node_tile + in.local_invocation_id.x + in.local_invocation_id.y * 2u + in.local_invocation_id.z * 4u;
        
        var child_shift = vec3<f32>(in.local_invocation_id) - 0.5; // (0,0,0) - (1,1,1) => (-0.5,-0.5,-0.5) - (0.5,0.5,0.5)
        child_shift = child_shift * 0.5;                                           // (-0.5,-0.5,-0.5) - (0.5,0.5,0.5) => (-0.25,-0.25,-0.25) - (0.25,0.25,0.25)
        child_shift = bounding_cube_transform(parent_node.vertex, child_shift);
        
        node_vertices[node_index] = vec4(child_shift, parent_node.vertex.w * 0.5);
    }
    
    workgroupBarrier(); // synch updateing node_vertices buffer
}

/// !!! whole workgroup must enter !!!
fn process_node(in: ShaderInput, node: Node) {
    var is_subdivided = 0u;
    var has_brick = 0u;
    var tile_index = 0u;
    
    let brick_evalutaion_result = evaluate_node_brick(in, node);
    if (brick_evalutaion_result.brick_type == BRICK_IS_BOUONDARY) {
        has_brick = 1u;
        if (brick_evalutaion_result.voxel_size > work_assigment.voxel_size) {
            is_subdivided = 1u;
            tile_index = create_tile(in);
            if (tile_index != 0u) {
                initialize_tile(in, node, tile_index);
            }
        }
    }
    
    // Update node buffers
    if (in.local_invocation_index == 0u) {
        // link node to its tile
        node_headers[node.index] = create_node_header(tile_index, is_subdivided, has_brick);
        
        // set payload value (brick coords or full/empty flag)
        if (has_brick == 1u) {
            node_payload[node.index] = create_node_brick_payload(brick_evalutaion_result.brick_location);
        } else {
            node_payload[node.index] = brick_evalutaion_result.brick_type;
        }
    }
    workgroupBarrier();
}

/// !!! Enter only with single workgroup !!!
fn process_root(in: ShaderInput) {
    let node = Node(0u, 0u, 0u, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    let brick_evalutaion_result = evaluate_node_brick(in, node);
    let tile_index = create_tile(in);
    initialize_tile(in, node, tile_index);
    // No need to write brick location anywhere, for rott it is always (0,0,0)
}

@compute
@workgroup_size(8, 8, 8)
fn main(in: ShaderInput) {
    let workgroup_index = in.workgroup_id.x + in.workgroup_id.y * in.num_workgroups.x + in.workgroup_id.z * in.num_workgroups.x * in.num_workgroups.y;
    let thread_zero = workgroup_index == 0u && in.local_invocation_index == 0u;
    let start_index = work_assigment.start_index;
    
    if (work_assigment.is_root == 1u) {
        if (workgroup_index == 0u) {
            process_root(in);
        }
    } else {
        let node = load_node(start_index + workgroup_index);
        process_node(in, node);
    }
}
