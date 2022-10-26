
struct PushConstants {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
}
var<push_constant> pc: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>
}

struct InstanceInput {
    @location(1) node_index: u32
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

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) in_face_position: vec4<f32>,
    // @location(1) @interpolate(flat) brick_local_camera_pos: vec4<f32>,
    // @location(2) @interpolate(flat) brick_to_local_transform_1: vec4<f32>,
    // @location(3) @interpolate(flat) brick_to_local_transform_2: vec4<f32>,
    // @location(4) @interpolate(flat) brick_to_local_transform_3: vec4<f32>,
    // @location(5) @interpolate(flat) brick_to_local_transform_4: vec4<f32>,
};

@vertex
fn vs_main(vertex_input: VertexInput, instance_input: InstanceInput) -> VertexOutput {
    let node_vertex = node_vertices[instance_input.node_index];
    var out: VertexOutput;
    var position = vec4<f32>(
        node_vertex.xyz + (node_vertex.w * vertex_input.position),
        1.0
    );
    out.position = pc.view_projection * position;
    out.in_face_position = position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.in_face_position;
}
