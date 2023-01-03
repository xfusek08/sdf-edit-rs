use wgpu::util::DeviceExt;

use crate::{
    framework::{
        self,
        math::Transform,
    },
};

#[derive(Debug)]
pub struct Camera {
    pub binding: u32,
    pub projection_matrix: glam::Mat4,
    pub focal_length: f32,
    pub transform: Transform,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstantData {
    pub projection_matrix: glam::Mat4,
    pub position:          glam::Vec4,
}

impl Camera {
    
    #[profiler::function]
    pub fn new(binding: u32, device: &wgpu::Device) -> Self {
        
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_uniform_buffer"),
                contents: bytemuck::cast_slice(&[0.0; 16]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding,
                    resource: uniform_buffer.as_entire_binding()
                }
            ]
        });
        
        Self {
            projection_matrix: glam::Mat4::IDENTITY,
            focal_length: 1.0,
            transform: Transform::default(),
            binding,
            bind_group_layout,
            bind_group,
            uniform_buffer,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, queue: &wgpu::Queue, camera: &framework::camera::Camera) {
        self.transform = camera.transform();
        self.projection_matrix = camera.view_projection_matrix();
        self.focal_length = camera.focal_length();
        profiler::call!(
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.projection_matrix]))
        );
    }
    
    #[profiler::function]
    pub fn to_push_constant_data(&self) -> PushConstantData {
        PushConstantData {
            projection_matrix: self.projection_matrix,
            position: glam::Vec4::from((self.transform.position, 1.0)),
        }
    }
    
}
