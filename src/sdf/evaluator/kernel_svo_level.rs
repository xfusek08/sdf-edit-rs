///!
///! An abstraction representing a kernel that can be dispatched to evaluate a level of an SVO.
///!

use std::borrow::Cow;

use crate::{
    sdf::svo,
    framework::{ gpu, math },
};

use super::{EvaluationContext, EvaluationContextLayouts};

/// An owning struct representing a context set for a kernel.
///   - When kernel has this context it means it is able to dispatch itself and evaluate SVO data on GPU.
///   - Context is set from outside defining what and how to evaluate.
struct KernelContext {
    evaluation_context: EvaluationContext,
    domain: math::BoundingCube,
    minium_voxel_size: f32,
}

///
/// An abstraction itself representing a kernel that can be dispatched to evaluate a level of an SVO.
///
pub struct KernelSVOLevel {
    
    /// An internal struct representing a context set for a kernel. See KernelContext for more info.
    context: Option<KernelContext>,
    
    /// A compute pipeline used to dispatch the kernel.
    pipeline: wgpu::ComputePipeline,
        
    /// A uniform buffer used to pass assignment data to the kernel.
    assignment_uniform: AssignmentUniform,
    
    /// A uniform buffer used to pass brick padding indices to the kernel.
    brick_padding_indices_uniform: BrickPaddingIndicesUniform,
}

// Public API
impl KernelSVOLevel {
    
    #[profiler::function]
    pub fn new(gpu: &gpu::Context) -> KernelSVOLevel {
        let assignment_uniform = AssignmentUniform::new(gpu);
        let brick_padding_indices_uniform = BrickPaddingIndicesUniform::new(gpu);
        let context_layout = EvaluationContextLayouts::new(gpu);
        
        Self {
            pipeline: Self::create_pipeline(
                gpu,
                &context_layout,
                &assignment_uniform,
                &brick_padding_indices_uniform,
            ),
            assignment_uniform,
            brick_padding_indices_uniform,
            context: None,
        }
    }
    
    pub fn set_context(&mut self, evaluation_context: EvaluationContext, domain: math::BoundingCube, minium_voxel_size: f32) {
        self.context = Some(KernelContext {
            evaluation_context,
            domain,
            minium_voxel_size,
        });
    }
    
    pub fn take_context(&mut self) -> Option<EvaluationContext> {
        let context = self.context.take()?;
        Some(context.evaluation_context)
    }
    
    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }
    
    
    /// Returns next unevaluated level.
    #[profiler::function]
    pub fn evaluate_root(&mut self, gpu: &gpu::Context) -> svo::Level {
        let context = self.context.as_ref().expect("Kernel context is not set");
        let assignment = Assignment {
            start_index: 0,
            is_root: 1,
            domain: context.domain,
            minium_voxel_size: context.minium_voxel_size,
            _padding: 0,
        };
        self.evaluate(gpu, 1, 0, assignment)
    }
    
    /// Returns next unevaluated level.
    #[profiler::function]
    pub fn evaluate_level(&mut self, gpu: &gpu::Context, level: &svo::Level) -> svo::Level{
        let context = self.context.as_mut().expect("Kernel context is not set");
        let node_count = context.evaluation_context.svo.node_pool.load_count(gpu);
        let assignment = Assignment {
            start_index: level.start_index,
            is_root: 0,
            domain: context.domain,
            minium_voxel_size: context.minium_voxel_size,
            _padding: 0,
        };
        self.evaluate(gpu, level.node_count, node_count, assignment)
    }
}



// =================================================================================================
// Private Implementation
// =================================================================================================

impl KernelSVOLevel {
    
    #[profiler::function]
    fn create_pipeline(
        gpu: &gpu::Context,
        context_layouts: &EvaluationContextLayouts,
        assignment_uniform: &AssignmentUniform,
        brick_padding_indices_uniform: &BrickPaddingIndicesUniform,
    ) -> wgpu::ComputePipeline {
        
        let pipeline_layout = {
            profiler::scope!("KernelSVOLevel: Create Pipeline Layout");
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("KernelSVOLevel Pipeline Layout"),
                bind_group_layouts: &[
                    &context_layouts.node_pool,                       // 0
                    &context_layouts.brick_pool,                      // 1
                    &context_layouts.edits,                  // 2
                    &assignment_uniform.bind_group_layout,            // 3
                    &brick_padding_indices_uniform.bind_group_layout, // 4
                ],
                push_constant_ranges: &[],
            })
        };
        
        {
            profiler::scope!("KernelSVOLevel: Create Pipeline");
            gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("KernelSVOLevel Compute Pipeline"),
                layout: Some(&pipeline_layout),
                entry_point: "main",
                module: &gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("SVO Evaluator Compute Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("kernel_svo_level.wgsl"))),
                }),
            })
        }
    }
    
    /// Returns the next unevaluated level.
    fn evaluate(
        &mut self,
        gpu: &gpu::Context,
        to_evaluate_node_count: u32,
        current_node_count: u32,
        assignment: Assignment
    ) -> svo::Level {
        
        // Ensure context is set (panic if not)
        let context = self.context.as_mut().expect("Kernel context is not set");
        
        // Get more consistent access to node_pool and bind_groups from the context
        let  EvaluationContext {
            bind_groups,
            svo: svo::Svo { node_pool, .. },
            ..
        } = &mut context.evaluation_context;
        
        // Update the assignment uniform
        self.assignment_uniform.update(gpu, &assignment);
        
        // Create command encoder
        let mut encoder = {
            profiler::scope!("Level Evaluator: Creating command encoder");
            gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("KernelSVOLevel: Command Encoder"),
            })
        };
        
        // Create compute pass
        let mut compute_pass = {
            profiler::scope!("Level Evaluator: Creating Compute Pass");
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("KernelSVOLevel: Compute Pass"),
            })
        };
        
        { profiler::scope!("Level Evaluator: Setting pipeline");
            compute_pass.set_pipeline(&self.pipeline);
        }
        
        { profiler::scope!("Level Evaluator: Settings bind groups");
            compute_pass.set_bind_group(0, &bind_groups.node_pool, &[]);
            compute_pass.set_bind_group(1, &bind_groups.brick_pool, &[]);
            compute_pass.set_bind_group(2, &bind_groups.edits, &[]);
            compute_pass.set_bind_group(3, &self.assignment_uniform.bind_group, &[]);
            compute_pass.set_bind_group(4, &self.brick_padding_indices_uniform.bind_group, &[]);
        }
        
        { profiler::scope!("Level Evaluator: Dispatch");
            compute_pass.dispatch_workgroups(to_evaluate_node_count, 1, 1);
        }
        
        // End compute pass to allow command encoder to be submitted
        drop(compute_pass);
        
        // Submit command encoder to queue
        gpu.queue.submit(Some(encoder.finish()));
        
        { profiler::scope!("Level Evaluator: Wait for queue to finish computation");
            gpu.device.poll(wgpu::Maintain::Wait);
        }
        
        // Read node count from buffer and calculate newly created level
        node_pool.buffers_changed();
        let new_node_count = node_pool.load_count(gpu);
        
        // Return next unevaluated level
        let added_node_count = new_node_count - current_node_count;
        if assignment.is_root == 1 {
            svo::Level {
                start_index: 0,
                node_count: added_node_count,
            }
        } else {
            svo::Level {
                start_index: assignment.start_index + to_evaluate_node_count,
                node_count: added_node_count,
            }
        }
        
    }
    
}


// =================================================================================================
// Internal structs
// =================================================================================================



///
/// An internal struct meant to be uploaded to GPU uniform buffer containing specification of dispatched work.
///
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Assignment {
    // Bounding cube of the SVO evaluation domain. SVO will be fitted into this cube.
    domain: math::BoundingCube,
    
    /// Minimum voxel size in world space - svo will be divided until voxel size is smaller than this value
    minium_voxel_size: f32,
    
    /// If 1 then shader will evaluate as and only root brick creating first tile
    is_root: u32,
    
    /// Index of first node of first unevaluated tile which is to be evaluated
    start_index: u32,
    
    /// Padding
    _padding: u32,
}



///
/// An internal struct representing gpu layout and binding of Assignment for this kernel.
///
struct AssignmentUniform {
    buffer: gpu::Buffer<Assignment>,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl AssignmentUniform {
    
    #[profiler::function]
    pub fn new(gpu: &gpu::Context) -> Self {
        let buffer = gpu::Buffer::new_empty(
            &gpu,
            Some("KernelSVOLevel: Assignment Uniform Buffer"),
            1,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("KernelSVOLevel: Assignment Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("KernelSVOLevel: Assignment Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.buffer.as_entire_binding(),
            }],
        });
        
        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
    
    pub fn update(&mut self, gpu: &gpu::Context, assignment: &Assignment) {
        self.buffer.queue_update(gpu, &[*assignment]);
    }
}


///
/// An internal struct representing gpu layout and binding of brick padding indices for this kernel.
///
struct BrickPaddingIndicesUniform {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    _buffer: gpu::Buffer<glam::UVec4>,
}

impl BrickPaddingIndicesUniform {
    
    #[profiler::function]
    pub fn new(gpu: &gpu::Context) -> Self {
        let padding_indices = Self::generate_indices();
        
        let buffer = gpu::Buffer::new(
            gpu,
            Some("KernelSVOLevel: Brick Padding Indices Uniform Buffer"),
            &padding_indices,
            wgpu::BufferUsages::UNIFORM
        );
        
        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("KernelSVOLevel: Brick Padding Indices Bind Group Layout"),
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
            label: Some("KernelSVOLevel: Brick Padding Indices Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.buffer.as_entire_binding(),
            }],
        });
        
        Self {
            _buffer: buffer,
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
