
use crate::framework::{math::Transform, gpu};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUGeometryTransform {
    position: glam::Vec3,
    scale:    f32,
    rotation: glam::Quat,
}

impl GPUGeometryTransform {
    #[profiler::function]
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            position: transform.position,
            scale:    transform.scale.max_element(), // NOTE: Scaling will not be possible in any direction separately
            rotation: transform.rotation,
        }
    }
}

#[derive(Debug)]
pub struct GPUGeometryTransforms {
    pub transforms: gpu::Buffer<GPUGeometryTransform>,
    pub count:      gpu::Buffer<u32>,
}

impl GPUGeometryTransforms {
    fn map_transforms(transforms: &[Transform]) -> Vec<GPUGeometryTransform> {
        transforms.iter()
            .map(GPUGeometryTransform::from_transform)
            .collect::<Vec<_>>()
    }
    
    #[profiler::function]
    pub fn from_transforms(gpu: &gpu::Context, transforms: &[Transform]) -> Self {
        let transforms = Self::map_transforms(transforms);
        Self {
            transforms: gpu::Buffer::new(gpu, Some("Geometry transforms"), &transforms, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            count:      gpu::Buffer::new(gpu, Some("Geometry transforms count"), &[transforms.len() as u32], wgpu::BufferUsages::UNIFORM  | wgpu::BufferUsages::COPY_DST),
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, gpu: &gpu::Context, transforms: &[Transform]) {
        let transforms = Self::map_transforms(transforms);
        self.transforms.queue_update(gpu, &transforms);
        self.count.queue_update(gpu, &[transforms.len() as u32]);
    }
    
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &gpu::Context, stages: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Geometry transforms bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: stages,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: stages,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
    
    #[profiler::function]
    pub fn create_bind_group(&self, gpu: &gpu::Context, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Geometry transforms bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.transforms.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.count.buffer.as_entire_binding(),
                },
            ],
        })
    }
}
