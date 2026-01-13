use super::{Edit, Operation, Primitive};
use crate::framework::{gpu, math::AABBAligned};

// =================================================================================================
// GPU Edit
// =================================================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUEdit {
    /// Top 16 bits are the operation type and bottom 16 bits are the primitive type
    color: glam::Vec4,
    operation_primitive: u32,
    blending: f32,
    _padding: [u32; 2],
}

impl GPUEdit {
    pub fn new(
        operation: Operation,
        primitive: Primitive,
        color: glam::Vec4,
        blending: f32,
    ) -> Self {
        Self {
            operation_primitive: (operation.to_index()) << 16 | (primitive.to_index()),
            blending,
            color,
            _padding: [0; 2],
        }
    }
    pub fn from_edit(edit: &Edit) -> Self {
        Self::new(
            edit.operation.clone(),
            edit.primitive.clone(),
            edit.color,
            edit.blending,
        )
    }
}

// =================================================================================================
// GPU Edit Data
// =================================================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUEditData {
    /// Inverted transform for position ray sample the primitive - no scaling is supported
    transform_inverse: glam::Mat4,
    /// A variable data required by primitive
    dimensions: [f32; 4],
}

impl GPUEditData {
    pub fn new(transform_inverse: glam::Mat4, dimensions: [f32; 4]) -> Self {
        Self {
            transform_inverse,
            dimensions,
        }
    }

    pub fn from_edit(edit: &Edit) -> Self {
        Self::new(
            edit.transform.as_mat().inverse(),
            edit.primitive.dimension_data(),
        )
    }
}

// =================================================================================================
// GPU Edit List
// =================================================================================================

pub struct GPUEdits {
    pub edits: gpu::Buffer<GPUEdit>,
    pub edit_data: gpu::Buffer<GPUEditData>,
    pub aabbs: gpu::Buffer<AABBAligned>,
    pub count: gpu::Buffer<u32>,
}

// Constructors
impl GPUEdits {
    #[profiler::function]
    pub fn from_edit_list(gpu: &gpu::Context, edits: &[Edit]) -> Self {
        let (gpu_edits, gpu_edit_data, aabbs) = Self::map_data(edits);

        Self {
            edits: gpu::Buffer::new(
                gpu,
                Some("Geometry edits"),
                &gpu_edits,
                wgpu::BufferUsages::STORAGE,
            ),
            edit_data: gpu::Buffer::new(
                gpu,
                Some("Geometry edit Data"),
                &gpu_edit_data,
                wgpu::BufferUsages::STORAGE,
            ),
            aabbs: gpu::Buffer::new(
                gpu,
                Some("Geometry edit AABBs"),
                &aabbs,
                wgpu::BufferUsages::STORAGE,
            ),
            count: gpu::Buffer::new(
                gpu,
                Some("Geometry edit count"),
                &[edits.len() as u32],
                wgpu::BufferUsages::UNIFORM,
            ),
        }
    }
}

// Bind Groups
impl GPUEdits {
    #[profiler::function]
    pub fn create_bind_group_layout(
        gpu: &gpu::Context,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayout {
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Geometry edits bind group layout"),
                entries: &[
                    // Buffer with edits
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Buffer with edits data
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Buffer with edits AABBs
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Buffer with edits count
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility,
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
    pub fn create_bind_group(
        &self,
        gpu: &gpu::Context,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Geometry edits bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.edits.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.edit_data.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.aabbs.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.count.buffer.as_entire_binding(),
                },
            ],
        })
    }
}

// Private
impl GPUEdits {
    #[profiler::function]
    fn map_data(edits: &[Edit]) -> (Vec<GPUEdit>, Vec<GPUEditData>, Vec<AABBAligned>) {
        let mut gpu_edits = vec![];
        let mut gpu_edit_data = vec![];
        let mut aabbs = vec![];

        for edit in edits {
            gpu_edits.push(GPUEdit::from_edit(edit));
            gpu_edit_data.push(GPUEditData::from_edit(edit));
            aabbs.push(AABBAligned::from_aabb(&edit.aabb()));
        }

        (gpu_edits, gpu_edit_data, aabbs)
    }
}
