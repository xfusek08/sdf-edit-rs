///! Module providing a GPU uniform immutable resource which enumerates exactly 488 3d indices of the surface crust of a 10x10x10 brick.
///! It is meant to efficiently assign gpu threads in 8x8x8 workgroup to evaluate a SDF brick padding after the brick is evaluated.

use crate::framework::gpu;

pub struct BrickPaddingIndicesUniform {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    uniform_buffer: gpu::Buffer<glam::UVec4>,
}

impl BrickPaddingIndicesUniform {
    #[profiler::function]
    pub fn new(gpu: &gpu::Context) -> Self {
        let padding_indices = Self::generate_indices();
        
        let uniform_buffer = gpu::Buffer::new(
            gpu,
            Some("Brick Padding Indices Uniform Buffer"),
            &padding_indices,
            wgpu::BufferUsages::UNIFORM
        );
        
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Brick Padding Indices Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });
        
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Brick Padding Indices Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.buffer.as_entire_binding(),
            }],
        });
        
        Self {
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }
    
    fn generate_indices() -> [glam::UVec4; 488] {
        let mut indices = [glam::UVec4::ZERO; 488];
        let mut i = 0;
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    if x == 0 || x == 9 || y == 0 || y == 9 || z == 0 || z == 9 {
                        indices[i] = glam::UVec4::new(x, y, z, 0);
                        i += 1;
                    }
                }
            }
        }
        indices
    }
}
