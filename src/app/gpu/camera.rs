use wgpu::util::DeviceExt;

use crate::app::{
    camera::Camera,
    math::Transform
};

#[derive(Debug)]
pub struct GPUCamera {
    pub binding: u32,
    pub view: glam::Mat4,
    pub transform: Transform,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUCameraPushConstantData {
    view:     glam::Mat4,
    position: glam::Vec4,
}

impl GPUCamera {
    
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
            view: glam::Mat4::IDENTITY,
            transform: Transform::default(),
            binding,
            bind_group_layout,
            bind_group,
            uniform_buffer,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.transform = camera.transform();
        self.view = camera.view_projection_matrix();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.view]));
    }
    
    #[profiler::function]
    pub fn to_push_constant_data(&self) -> GPUCameraPushConstantData {
        GPUCameraPushConstantData {
            view: self.view,
            position: glam::Vec4::from((self.transform.position, 1.0)),
        }
    }
    
}
