
struct PushConstants {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
}
var<push_constant> pc: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>
}

struct InstanceInput {
    @location(1) node_vertex: vec4<f32>
}


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(vertex_input: VertexInput, instance_input: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    var position = instance_input.node_vertex.xyz + (instance_input.node_vertex.w * vertex_input.position);
    out.position = pc.view_projection * vec4<f32>(position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.2);
}
