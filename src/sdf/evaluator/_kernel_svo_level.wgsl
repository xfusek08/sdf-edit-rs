
struct ShaderInput {
    @builtin(num_workgroups)         num_workgroups:         vec3<u32>,
    @builtin(workgroup_id)           workgroup_id:           vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(local_invocation_id)    local_invocation_id:    vec3<u32>,
}


// =================================================================================================
// Bind group 0: SVO: Node pool
// =================================================================================================

@group(0) @binding(0) var<storage, read_write> node_count:         atomic<u32>; // number of nodes in tiles buffer, use to atomically add new nodes
@group(0) @binding(1) var<storage, read_write> node_headers:       array<u32>;
@group(0) @binding(2) var<storage, read_write> node_payload:       array<u32>;
@group(0) @binding(3) var<storage, read_write> node_vertices:      array<vec4<f32>>;
@group(0) @binding(4) var<uniform>             node_pool_capacity: u32; // maximum number of nodes in tiles buffer

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

// TODO: Use preprocessor for constatns
let HEADER_IS_SUBDIVIDED_SHIFT = 31u;
let HEADER_HAS_BRICK_SHIFT = 30u;
let HEADER_TILE_INDEX_MASK = 0x3FFFFFFFu;

/// Combines tile index and flags into single node header integer
///   - `is_subdivided` must have value 0 or 1
///   - `is_leaf` must have value 0 or 1
fn create_node_header(value: u32, is_subdivided: u32, has_brick: u32) -> u32 {
    return (value & HEADER_TILE_INDEX_MASK) | (is_subdivided << HEADER_IS_SUBDIVIDED_SHIFT) | (has_brick << HEADER_HAS_BRICK_SHIFT);
}

/// Encodes brick location into single integer
fn create_node_brick_payload(brick_location: vec3<u32>) -> u32 {
    return ((brick_location.x & 0x3FFu) << 20u) | ((brick_location.y & 0x3FFu) << 10u) | (brick_location.z & 0x3FFu);
}


// =================================================================================================
// Bind group 1: SVO: Brick pool
// =================================================================================================

@group(1) @binding(0) var brick_atlas: texture_storage_3d<r32float, write>;
@group(1) @binding(1) var<storage, read_write> brick_count: atomic<u32>; // number of bricks in brick texture, use to atomically add new bricks
@group(1) @binding(2) var<uniform> brick_pool_side_size: u32;            // Number of bricks in one side of the brick atlas texture

/// Converts brick index to brick location in brick atlas texture
fn brick_index_to_coords(index: u32) -> vec3<u32> {
    var side_size = brick_pool_side_size;
    return  vec3<u32>(
        index % side_size,
        (index / side_size) % side_size,
        (index / side_size) / side_size
    );
}


// =================================================================================================
// Bind group 2: Edit List represented SDF which will be sampled
//      - Will be iterate over and over for each voxel in each node
//      - NOTE: Maybe use uniform buffer when there are not too many items
//      - NOTE: Use BVH or Octree representation of edits for faster iteration
// =================================================================================================

// TODO: Use preprocessor for constatns
let EDIT_PRIMITIVE_SPHERE = 0u;
let EDIT_PRIMITIVE_CUBE = 1u;
let EDIT_PRIMITIVE_CYLINDER = 2u;
let EDIT_PRIMITIVE_TORUS = 3u;
let EDIT_PRIMITIVE_CONE = 4u;
let EDIT_PRIMITIVE_CAPSULE = 5u;

// TODO: Use preprocessor for constatns
let EDIT_OPERATION_ADD = 0u;
let EDIT_OPERATION_SUBTRACT = 1u;
let EDIT_OPERATION_INTERSECT = 2u;
// ...

struct EditPacked {
    operation_primitive: u32,
    blending: f32,
}

struct Edit {
    operation: u32,
    primitive: u32,
    blending: f32,
}

fn unpack_edit(packed_edit: EditPacked) -> Edit {
    return Edit(
        packed_edit.operation_primitive >> 16u,
        packed_edit.operation_primitive & 0xFFFFu,
        packed_edit.blending
    );
}

struct EditData {
    transform: mat4x4<f32>,
    dimensions: vec4<f32>,
}

struct AABB {
    min: vec3<f32>,
    padding1: f32,
    max: vec3<f32>,
    padding2: f32,
}

fn in_aabb(position: vec3<f32>, aabb: AABB) -> bool {
    return all(position >= aabb.min) && all(position <= aabb.max);
}

@group(2) @binding(0) var<storage, read> edits:      array<EditPacked>;
@group(2) @binding(1) var<storage, read> edit_data:  array<EditData>;
@group(2) @binding(2) var<storage, read> edit_aabbs: array<AABB>;
@group(2) @binding(3) var<uniform>       edit_count: u32;


// =================================================================================================
// Bind group 3: Assigment uniform data
// =================================================================================================

struct Assigment {
    svo_boundding_cube: vec4<f32>, // bounding cube of the SVO in world space (xzy, distance from center to side)
    minium_voxel_size:  f32,       // minimum voxel size in world space - divide node if its voxels are bigger then this value
    is_root:            u32,       // is this the root node? [0/1]
    start_index:        u32,       // node index from which to start the evaluation
}
@group(3) @binding(0) var<uniform> assigment: Assigment;


// =================================================================================================
// Bind group 4: SVO: Brick padding indices for indexing brick paddings
//      (TODO: might be replaced by over-extending sampled points and storing into tightly packed 8x8x8 bricks)
// =================================================================================================

struct BrickPaddingIndices {
    data: array<vec3<u32>, 488>
}
@group(4) @binding(0) var<uniform> brick_padding_indices: BrickPaddingIndices;


// =================================================================================================
// General Functions
// =================================================================================================

fn bounding_cube_transform(bc: vec4<f32>, position: vec3<f32>) -> vec3<f32> {
    return bc.w * position + bc.xyz;
}


// =================================================================================================
//                                              SDF Sampling
// =================================================================================================

fn transform_pos(edit: EditData, position: vec3<f32>) -> vec3<f32> {
    return (edit.transform * vec4<f32>(position, 1.0)).xyz;
}

fn sd_shpere(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position);
    // let p = position;
    return length(p) - edit_data.dimensions.x;
}

fn sd_cube(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position);
    let d = abs(p) - edit_data.dimensions.xyz + edit_data.dimensions.w;
    let e = length(max(d, vec3(0.0)));
    let i = min(max(d.x, max(d.y, d.z)), 0.0);
    return e + i - edit_data.dimensions.w;
}

fn sd_cylinder(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position);
    let w = edit_data.dimensions[0] - edit_data.dimensions[2];
    let h = edit_data.dimensions[1] - edit_data.dimensions[2];
    let d = abs(vec2(length(p.xz), p.y)) - vec2(w, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2(0.0))) - edit_data.dimensions[2];
}

fn sd_torus(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position);
    let x = length(p.xz) - edit_data.dimensions[0];
    return length(vec2(x, p.y)) - edit_data.dimensions[1];
}

fn sd_cone(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position) - vec3(0.0, edit_data.dimensions[1], 0.0);
    let c = vec2(edit_data.dimensions[1], edit_data.dimensions[0]);
    let h = edit_data.dimensions[1] * 2.0;
    let q = length(p.xz);
    return max(dot(c, vec2(q, p.y)), -h - p.y);
}

fn sd_capsule(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    let p = transform_pos(edit_data, position);
    let h = edit_data.dimensions[1];
    let r = edit_data.dimensions[0];
    let p = p + vec3(0.0, h * 0.5, 0.0);
    let p = p - vec3(0.0, clamp(p.y, 0.0, h), 0.0);
    return length(p) - r;
}

fn smooth_min(dist1: f32, dist2: f32, koeficient: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (dist2 - dist1) / koeficient, 0.0, 1.0);
    return mix(dist2, dist1, h) - koeficient * h * (1.0 - h);
}

fn smooth_max(dist1: f32, dist2: f32, koeficient: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (dist1 - dist2) / koeficient, 0.0, 1.0);
    return mix(dist1, dist2, h) + koeficient * h * (1.0 - h);
}

fn distance_to_edit(position: vec3<f32>, edit: Edit, edit_data: EditData) -> f32 {
    
    // TODO Use preprocessor because constant are not yet supported in naga
    switch (edit.primitive) {
        // EDIT_PRIMITIVE_SPHERE
        case 0u: { return sd_shpere(position, edit, edit_data); }
        // EDIT_PRIMITIVE_CUBE
        case 1u: { return sd_cube(position, edit, edit_data); }
         // EDIT_PRIMITIVE_CYLINDER
        case 2u: { return sd_cylinder(position, edit, edit_data); }
        // EDIT_PRIMITIVE_TORUS
        case 3u: { return sd_torus(position, edit, edit_data); }
        // EDIT_PRIMITIVE_CONE
        case 4u: { return sd_cone(position, edit, edit_data); }
        // EDIT_PRIMITIVE_CAPSULE
        case 5u: { return sd_capsule(position, edit, edit_data); }
        // Default to make the compiler happy
        default: {
            return 1000000.0;
        }
    }
}

fn sample_sdf(position: vec3<f32>) -> f32 {
    // var was_in_aabb = false;
    var sdf_value = 1000000.0;
    for (var i = 0u; i < edit_count; i = i + 1u) {
        let aabb = edit_aabbs[i];
        
        let edit = unpack_edit(edits[i]);
        let distance_to_primitive = distance_to_edit(position, edit, edit_data[i]);
        
        // TODO Use preprocessor because constant are not yet supported in naga
        switch (edit.operation) {
            // EDIT_OPERATION_ADD
            case 0u: { sdf_value = smooth_min(sdf_value, distance_to_primitive, edit.blending); }
            // EDIT_OPERATION_SUBTRACT
            case 1u: { sdf_value = smooth_max(sdf_value, -distance_to_primitive, edit.blending); }
            // EDIT_OPERATION_INTERSECT
            case 2u: { sdf_value = smooth_max(sdf_value, distance_to_primitive, edit.blending); }
            
            default: {} // to make naga happy
        }
    }
    return sdf_value;
}


// =================================================================================================
// Node Evaluation into brick
// =================================================================================================

// TODO: Use preprocessor for constatns
let BRICK_IS_EMPTY = 0u;
let BRICK_IS_BOUONDARY = 1u;
let BRICK_IS_FILLED = 2u;

var<workgroup> divide: atomic<u32>;
var<workgroup> brick_index: u32;

struct BrickEvaluationResult {
    brick_type: u32,
    voxel_size: f32,
    brick_location: vec3<u32>,
}

struct GlobalVoxelDesc {
    center: vec3<f32>,
    size: f32,
}

fn calculate_global_voxel(centered_voxel_index: vec3<i32>, node: Node) -> GlobalVoxelDesc {
    let voxel_size = 0.125; // 1.0 / 8.0;
    let half_step = 0.0625; // voxel_size * 0.5;
    let shift_vector = voxel_size * vec3<f32>(centered_voxel_index) + half_step;
    let voxel_center_local = bounding_cube_transform(node.vertex, shift_vector);
    let voxel_center_global = bounding_cube_transform(assigment.svo_boundding_cube, voxel_center_local);
    let voxel_size_local = voxel_size * node.vertex.w;
    let voxel_size_global = voxel_size_local * assigment.svo_boundding_cube.w;
    return GlobalVoxelDesc(voxel_center_global, voxel_size_global);
}

fn write_to_brick(voxel_coords: vec3<i32>, distance: f32) {
    textureStore(brick_atlas, voxel_coords, vec4<f32>(distance, 0.0, 0.0, 0.0));
}

fn in_voxel(voxel_size: f32, dinstance: f32) -> bool {
    // TODO: use max-norm for evaluating this
    let sqrt_3 = 1.7320508075688772935274463415059;
    // let voxel_bounding_spehere_radius = (voxel_size * sqrt_3) * 0.6;
    let voxel_bounding_spehere_radius = (voxel_size * sqrt_3) * 1.5;
    return abs(dinstance) < voxel_bounding_spehere_radius;
}

// Main function of this section
fn evaluate_node_brick(in: ShaderInput, node: Node) -> BrickEvaluationResult {
    var result: BrickEvaluationResult;
    
    let centered_voxel_index = vec3<i32>(in.local_invocation_id) - 4; // (0,0,0) - (7,7,7) => (-4,-4,-4) - (3,3,3)
    let voxel_global_desc = calculate_global_voxel(centered_voxel_index, node);
    let sdf_value = sample_sdf(voxel_global_desc.center);
    
    // vote if voxel intersects sdf surface
    if (in.local_invocation_index == 0u) {
        atomicStore(&divide, 0u);
    }
    
    if (in_voxel(voxel_global_desc.size, sdf_value)) {
        atomicAdd(&divide, 1u);
    }
    workgroupBarrier(); // synchronize witing of whole group if to divide or not
    
    if (atomicLoad(&divide) > 0u) { // full workgroup branching
        // Save evaluated volume into a new brick
        
        // Take next brick index
        if (in.local_invocation_index == 0u) {
            brick_index = atomicAdd(&brick_count, 1u);
        }
        workgroupBarrier();  // synchronize allocation of brick index
        
        // All threads in group will find voxel coordinate in brick pool based on the brick index
        let brick_coords = brick_index_to_coords(brick_index);
        let brick_coords_10 = brick_coords * 10u;
        
        // Get coordinates of voxel in brick (10 = 8 + 2 padding)
        let voxel_coords = brick_coords_10 + in.local_invocation_id + 1u;
        
        // save voxel value
        write_to_brick(vec3<i32>(voxel_coords), sdf_value);
        
        // Write padding
        if (in.local_invocation_index < 488u) {
            let padding_index = brick_padding_indices.data[in.local_invocation_index];
            let centered_voxel_index = vec3<i32>(padding_index) - 5;
            let voxel_global_desc = calculate_global_voxel(centered_voxel_index, node);
            let sdf_value = sample_sdf(voxel_global_desc.center);
            let voxel_coords = brick_coords_10 + padding_index;
            write_to_brick(vec3<i32>(voxel_coords), sdf_value);
        }
        workgroupBarrier();  // wait for all threads to finish writing padding to brick
        
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
    
    result.voxel_size = voxel_global_desc.size;
    return result;
}


// =================================================================================================
// Node pool Tile management
// =================================================================================================

// Allocates a new tile and returns its index
var<workgroup> tile_index_shared: u32;
fn create_tile(in: ShaderInput) -> u32 {
    if (in.local_invocation_index == 0u) {
        tile_index_shared = 0u;
        if (atomicLoad(&node_count) < node_pool_capacity - 8u) {
            // tile might still exceed node pool capacity
            let first_tile_node_index = atomicAdd(&node_count, 8u);
            if (node_pool_capacity > (first_tile_node_index + 8u)) {
                tile_index_shared = first_tile_node_index >> 3u;
            } else {
                // Refuse to initialize the tile becauase there is no more capacity node count increment has to be corrected.
                atomicSub(&node_count, 8u); // TODO: This will not be needed when trimming incomplete levels
            }
        }
    }
    workgroupBarrier(); // synch tile_start_index value
    return tile_index_shared;
}

// Initializes a new tile by computing vertices for each node and writing them into node_vertices buffer
fn initialize_tile(in: ShaderInput, parent_node: Node, tile_index: u32) {
    
    var shift_vector: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
        vec3<f32>(-0.25, -0.25, -0.25),
        vec3<f32>(-0.25, -0.25,  0.25),
        vec3<f32>(-0.25,  0.25, -0.25),
        vec3<f32>(-0.25,  0.25,  0.25),
        vec3<f32>( 0.25,  0.25, -0.25),
        vec3<f32>( 0.25,  0.25,  0.25),
        vec3<f32>( 0.25, -0.25, -0.25),
        vec3<f32>( 0.25, -0.25,  0.25),
    );
    
    // Enters 2x2x2 subgroup of threads
    if (in.local_invocation_id.x < 8u) {
        let start_node_tile = tile_index << 3u;
        let node_index = start_node_tile + in.local_invocation_id.x;
        
        var child_shift = shift_vector[in.local_invocation_id.x]; // (0,0,0) - (1,1,1) => (-0.5,-0.5,-0.5) - (0.5,0.5,0.5)
        child_shift = bounding_cube_transform(parent_node.vertex, child_shift);
        
        node_vertices[node_index] = vec4(child_shift, parent_node.vertex.w * 0.5);
    }
    
    workgroupBarrier(); // synch updateing node_vertices buffer
}


// =================================================================================================
// Top level implementation of node processing
//      - evaluate node into brick
//      - create tile if needed
//      - initialize tile if needed
// =================================================================================================

// !!! whole workgroup must enter !!!
fn process_node(in: ShaderInput, node: Node) {
    var is_subdivided = 0u;
    var has_brick = 0u;
    var tile_index = 0u;
    
    let brick_evalutaion_result = evaluate_node_brick(in, node);
    if (brick_evalutaion_result.brick_type == BRICK_IS_BOUONDARY) {
        has_brick = 1u;
        if (brick_evalutaion_result.voxel_size > assigment.minium_voxel_size) {
            tile_index = create_tile(in);
            if (tile_index != 0u) {
                is_subdivided = 1u;
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

// !!! Enter only with single workgroup !!!
fn process_root(in: ShaderInput) {
    
    // Clear node pool by resetting node count
    if (in.local_invocation_index == 0u) {
        atomicStore(&node_count, 0u);
        atomicStore(&brick_count, 0u);
    }
    workgroupBarrier();
    
    // Create root node
    let node = Node(0u, 0u, 0u, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    
    // Evaluate root node
    let brick_evalutaion_result = evaluate_node_brick(in, node);
    
    // Prepare first tile (child of root node)
    let tile_index = create_tile(in);
    initialize_tile(in, node, tile_index);
    
    // No need to write brick location anywhere, for root it is always (0,0,0)
}


// =================================================================================================
// Entry point
// =================================================================================================

@compute
@workgroup_size(8, 8, 8)
fn main(in: ShaderInput) {
    let workgroup_index = in.workgroup_id.x + in.workgroup_id.y * in.num_workgroups.x + in.workgroup_id.z * in.num_workgroups.x * in.num_workgroups.y;
    let thread_zero = workgroup_index == 0u && in.local_invocation_index == 0u;
    let start_index = assigment.start_index;
    
    if (assigment.is_root == 1u) {
        if (workgroup_index == 0u) {
            process_root(in);
        }
    } else {
        let node = load_node(start_index + workgroup_index);
        process_node(in, node);
    }
}
