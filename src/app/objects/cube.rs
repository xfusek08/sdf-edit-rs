
use glam::Vec4Swizzles;
use wgpu::util::DeviceExt;

use crate::app::gpu::vertices::SimpleVertex;
use super::PRIMITIVE_RESTART;

pub const CUBE_VERTICES: &[SimpleVertex] = &[
    // front face
    SimpleVertex(glam::Vec3::new(-0.5,  0.5, 0.5)), // 0 TL
    SimpleVertex(glam::Vec3::new(-0.5, -0.5, 0.5)), // 1 BL
    SimpleVertex(glam::Vec3::new( 0.5,  0.5, 0.5)), // 2 TR
    SimpleVertex(glam::Vec3::new( 0.5, -0.5, 0.5)), // 3 BR
    
    // back face
    SimpleVertex(glam::Vec3::new(-0.5,  0.5, -0.5)), // 4 TL
    SimpleVertex(glam::Vec3::new(-0.5, -0.5, -0.5)), // 5 BL
    SimpleVertex(glam::Vec3::new( 0.5,  0.5, -0.5)), // 6 TR
    SimpleVertex(glam::Vec3::new( 0.5, -0.5, -0.5)), // 7 BR
];

pub const CUBE_INDICES_TRIANGLE_STRIP: &[u16] = &[
    0, 1, 2, 3, 6, 7, 4, 5,
    PRIMITIVE_RESTART,
    2, 6, 0, 4, 1, 5, 3, 7,
];

pub const CUBE_INDICES_LINE_STRIP: &[u16] = &[
    0, 1, 3, 7, 5, 1,
    PRIMITIVE_RESTART,
    5, 4, 0, 2, 6, 4,
    PRIMITIVE_RESTART,
    3, 2, 6, 7
];

/// Component which will be rendered as cube outline
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeOutlineComponent {
    pub data: glam::Vec4
}
impl CubeOutlineComponent {
    pub fn new(x: f32, y: f32, z: f32, size: f32) -> Self {
        Self { data: glam::Vec4::new(x,y,z, size) }
    }
    pub fn position(&self) -> glam::Vec3 {
        self.data.xyz()
    }
    pub fn size(&self) -> f32 {
        self.data.w
    }
    pub fn set_position(&mut self, position: glam::Vec3) {
        self.data.x = position.x;
        self.data.y = position.y;
        self.data.z = position.z;
    }
    pub fn set_size(&mut self, size: f32) {
        self.data.w = size;
    }
}

#[derive(Debug)]
pub struct CubeWireframeMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}
impl CubeWireframeMesh {
    #[profiler::function]
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&CUBE_INDICES_LINE_STRIP),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Debug)]
pub struct CubeSolidMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}
impl CubeSolidMesh {
    #[profiler::function]
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&CUBE_INDICES_TRIANGLE_STRIP),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}
