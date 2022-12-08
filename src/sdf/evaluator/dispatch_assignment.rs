
use crate::{
    framework::{
        gpu,
        math::BoundingCube,
    }
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DispatchAssignment {
    // Bounding cube of the SVO evaluation domain. SVO will be fitted into this cube.
    pub svo_bounding_cube: BoundingCube,
    
    /// Minimum voxel size in world space - svo will be divided until voxel size is smaller than this value
    pub min_voxel_size: f32,
    
    /// If 1 then shader will evaluate as and only root brick creating first tile
    pub is_root: u32,
    
    /// Index of first node of first unevaluated tile which is to be evaluated
    pub start_index: u32,
    
    /// Padding
    pub _padding: u32,
}

impl DispatchAssignment {
    pub fn new_root(svo_bounding_cube: BoundingCube, min_voxel_size: f32) -> Self {
        Self {
            svo_bounding_cube,
            min_voxel_size,
            is_root: 1,
            start_index: 0,
            _padding: 0,
        }
    }
}

/// A work assignment GPU resource
pub struct DispatchAssignmentUniform {
    
    /// Work assignment Data
    pub work_assignment: DispatchAssignment,
    
    /// This structure represented in uniform buffer on GPU
    pub uniform_buffer: gpu::Buffer<DispatchAssignment>,
}

// GPU binding
impl DispatchAssignmentUniform {
    #[profiler::function]
    pub fn new(gpu: &gpu::Context, work_assignment: DispatchAssignment) -> Self {
        let uniform_buffer = gpu::Buffer::new(
            gpu,
            Some("Work Assignment Uniform Buffer"),
            &[work_assignment],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ
        );
        Self {
            work_assignment,
            uniform_buffer,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, gpu: &gpu::Context, work_assignment: DispatchAssignment) {
        self.uniform_buffer.queue_update(gpu, &[work_assignment]);
        self.work_assignment = work_assignment;
    }
    
    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn create_bind_group(&mut self, gpu: &gpu::Context, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WorkAssignment Bind Group"),
            layout: layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.buffer.as_entire_binding(),
            }],
        })
    }
    
    /// Creates and returns a custom binding for the node pool.
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &gpu::Context, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Evaluator Work Assignment Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }
    
}
