use std::mem::size_of;
use glam::{Vec3, Vec2};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
}
// Return a vertex layout (configuration for vertex puller)
impl Vertex {
    
    // ⬇ also could be replaced by macro: `&wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],`
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        // ↓ Position
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        // ⬇ Color
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: size_of::<Vec3>() as wgpu::BufferAddress,
            shader_location: 1
        }
    ];
    
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress, // <- width of one vertex in the buffer (each vertex is contained of N bytes)
            step_mode: wgpu::VertexStepMode::Vertex, // <- If set to Instance - each vertex will be pulled once per instance
            attributes: Self::ATTRIBUTES,
        }
    }
}
