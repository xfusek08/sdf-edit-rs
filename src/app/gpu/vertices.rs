use std::mem::size_of;

use glam::Vec3;


/// A trait which each vertex type must implement.
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}

/// Simple Vertex
/// A vertex holding just a position.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex(pub Vec3);

impl Vertex for SimpleVertex {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        }
    }
}

/// Color Vertex
/// A vertex type which contains position and color.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: Vec3,
    pub color: Vec3,
}
impl ColorVertex {
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
            format: wgpu::VertexFormat::Float32x3,
            offset: size_of::<Vec3>() as wgpu::BufferAddress,
            shader_location: 1
        }
    ];
}
impl Vertex for ColorVertex {
    fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorVertex>() as wgpu::BufferAddress, // <- width of one vertex in the buffer (each vertex is contained of N bytes)
            step_mode: wgpu::VertexStepMode::Vertex, // <- If set to Instance - each vertex will be pulled once per instance
            attributes: ColorVertex::ATTRIBUTES,
        }
    }
}
