use std::mem::size_of;

use glam::{Vec3, Vec2};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: Vec3,
    tex_coords: Vec2,
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: Vec3::new( 0.0,  0.5, 0.0), tex_coords: Vec2::new(1.0, 0.0) },
    Vertex { position: Vec3::new(-0.5, -0.5, 0.0), tex_coords: Vec2::new(0.0, 1.0) },
    Vertex { position: Vec3::new( 0.5, -0.5, 0.0), tex_coords: Vec2::new(0.0, 0.0) },
];

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

pub const PENTAGON_VERTICES: &[Vertex] = &[
    Vertex { position: Vec3::new(-0.0868241, 0.49240386, 0.0), tex_coords: Vec2::new(0.4131759, 0.00759614) },
    Vertex { position: Vec3::new(-0.49513406, 0.06958647, 0.0), tex_coords: Vec2::new(0.0048659444, 0.43041354) },
    Vertex { position: Vec3::new(-0.21918549, -0.44939706, 0.0), tex_coords: Vec2::new(0.28081453, 0.949397) },
    Vertex { position: Vec3::new(0.35966998, -0.3473291, 0.0), tex_coords: Vec2::new(0.85967, 0.84732914) },
    Vertex { position: Vec3::new(0.44147372, 0.2347359, 0.0), tex_coords: Vec2::new(0.9414737, 0.2652641) },
];

pub const PENTAGON_INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
