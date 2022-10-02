
type V2 = vec2<f32>;
type V3 = vec3<f32>;
type V4 = vec4<f32>;

type M3 = mat3x3<f32>;
type M4 = mat4x4<f32>;

/// constructors are broken? : https://github.com/gfx-rs/naga/issues/1956
let V3_ZERO = V3(0.0, 0.0, 0.0);
let M4_IDENTITY = M4(
    V4(1.0, 0.0, 0.0, 0.0),
    V4(0.0, 1.0, 0.0, 0.0),
    V4(0.0, 0.0, 1.0, 0.0),
    V4(0.0, 0.0, 0.0, 1.0),
);

struct PushConstants {
    view_projection: M4,
    camera_position: V4,
}
var<push_constant> pc: PushConstants;

struct Ray {
    origin: V3,
    direction: V3,
    dist: f32,
};

// tmp
let node_vertex = V4(0.0, 0.0, 0.0, 1.0);
let transform_matrix = M4_IDENTITY;
// tmp end

fn translate(translation: V3) -> M4 {
    var res = M4_IDENTITY;
    res[3][0] = translation.x;
    res[3][1] = translation.y;
    res[3][2] = translation.z;
    return res;
}

fn scale(scaling: f32) -> M4 {
    var res = M4_IDENTITY;
    res[0][0] = scaling;
    res[1][1] = scaling;
    res[2][2] = scaling;
    return res;
}

// Founds length of ray until it exits the rendered cube
// Computed in ray-marcher space
// This function was inspired by:
//    https://medium.com/@bromanz/another-view-on-the-classic-ray-aabb-intersection-algorithm-for-bvh-traversal-41125138b525
//    - implemented reduced version of the efficient slab test algorithm
fn get_distance_to_end_of_brick(position: V3, direction: V3) -> f32 {
    
    // prepare bb of current box
    var maxCorner = V3(1.0, 1.0, 1.0);
    var minCorner = V3(0.0, 0.0, 0.0);
    
    var inverseRayDir = 1.0 / direction;
    var tMinV0 = (minCorner - position) * inverseRayDir;
    var tMaxV0 = (maxCorner - position) * inverseRayDir;
    
    var tMaxV = max(tMinV0, tMaxV0);
    
    return min(tMaxV.x, min(tMaxV.y, tMaxV.z));
}

struct VertexInput {
    @location(0) position: V3,
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    
    @location(0)
    frag_pos: V3,
    
    @location(1)
    brick_local_camera_pos: V4,
    
    @location(2) @interpolate(flat) brick_to_local_transform_1: V4,
    @location(3) @interpolate(flat) brick_to_local_transform_2: V4,
    @location(4) @interpolate(flat) brick_to_local_transform_3: V4,
    @location(5) @interpolate(flat) brick_to_local_transform_4: V4,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    var brick_position = node_vertex.xyz;
    var brick_inverted_size = 1.0 / node_vertex.w;
    var step_size = node_vertex.w * 0.5;
    var brick_shift = node_vertex.www * 0.5 - node_vertex.xyz;
    
    var position = brick_position + model.position;
    out.position = pc.view_projection * V4(model.position, 1.0);
    out.frag_pos = position;
    
    var brick_to_local_transform = scale(brick_inverted_size) * translate(brick_shift) * M4_IDENTITY;
    out.brick_to_local_transform_1 = brick_to_local_transform[0];
    out.brick_to_local_transform_2 = brick_to_local_transform[1];
    out.brick_to_local_transform_3 = brick_to_local_transform[2];
    out.brick_to_local_transform_4 = brick_to_local_transform[3];
    
    out.brick_local_camera_pos = (brick_to_local_transform * pc.camera_position);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) V4 {
    var brick_to_local_transform = M4(
        in.brick_to_local_transform_1,
        in.brick_to_local_transform_2,
        in.brick_to_local_transform_3,
        in.brick_to_local_transform_4,
    );
    var fragment_pos = (brick_to_local_transform * V4(in.frag_pos, 1.0)).xyz;
    
    // var ray: Ray;
    // ray.origin = fragment_pos;
    // ray.direction = normalize(fragment_pos - in.brick_local_camera_pos.xyz);
    // ray.dist = get_distance_to_end_of_brick(ray.origin, ray.direction);
    
    return V4(normalize(abs(fragment_pos)), 1.0);
    // return V4(normalize(in.brick_local_camera_pos.xyz) * 0.5, 1.0);
}
