
// evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{thread, sync::Arc, borrow::Cow};

use wgpu::util::DeviceExt;

use crate::{
    info,
    error,
    app::{
        gpu::GPUContext,
        math::{BoundingCube, AABB}
    },
};

use super::{
    svo,
    geometry::{
        Geometry,
        GeometryID,
        GeometryPool,
        GeometryEvaluationStatus,
        GeometryEditList,
    },
};

pub struct EvaluationJob {
    join_handle: thread::JoinHandle<svo::Octree>,
    geometry_id: GeometryID,
}

#[derive(Clone)]
pub struct EvaluationGPUResources {
    pub gpu: Arc<GPUContext>,
    pub pipeline: Arc<wgpu::ComputePipeline>,
    pub work_assignment_layout: Arc<wgpu::BindGroupLayout>,
    pub node_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub brick_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub dispatch_param_buffer_layout: Arc<wgpu::BindGroupLayout>,
}

pub struct Evaluator {
    gpu_resources: EvaluationGPUResources,
    evaluation_jobs: Vec<EvaluationJob>,
}

// when evaluator is dropped, it should wait for all evaluation threads to finish
impl Drop for Evaluator {
    #[profiler::function]
    fn drop(&mut self) {
        while let Some(job) = self.evaluation_jobs.pop() {
            job.join_handle.join().unwrap();
        }
    }
}

// Construction
impl Evaluator {
    #[profiler::function]
    pub fn new(gpu: Arc<GPUContext>) -> Evaluator {
        let work_assignment_layout = Arc::new(
            WorkAssignmentUniform::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let node_pool_bind_group_layout = Arc::new(
            svo::NodePool::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let brick_pool_bind_group_layout = Arc::new(
            svo::BrickPool::create_write_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let dispatch_param_buffer_layout = Arc::new(
            DispatchOutputBuffer::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        
        let pipeline_layout = { profiler::scope!("Create evaluator pipeline layout");
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[
                    work_assignment_layout.as_ref(),       // 0 - Work Assignment
                    node_pool_bind_group_layout.as_ref(),  // 1 - Node Pool
                    brick_pool_bind_group_layout.as_ref(), // 2 - Brick Pool
                    dispatch_param_buffer_layout.as_ref(), // 3 - Dispatch Params
                ],
                push_constant_ranges: &[],
            })
        };
        
        let pipeline = { profiler::scope!("Create evaluator pipeline");
            Arc::new(gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("SDF Evaluator"),
                layout: Some(&pipeline_layout),
                entry_point: "main",
                module: &gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("SVO Evaluator Compute Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/evaluate_svo_compute.wgsl"))),
                }),
            }))
        };
        
        Self {
            evaluation_jobs: vec![],
            gpu_resources: EvaluationGPUResources {
                gpu,
                pipeline,
                work_assignment_layout,
                node_pool_bind_group_layout,
                brick_pool_bind_group_layout,
                dispatch_param_buffer_layout,
            },
        }
    }
}

// Geometry evaluation job management (public interface)
impl Evaluator {
    
    #[profiler::function]
    pub fn evaluate_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        for (geometry_id, geometry) in geometry_pool.iter_mut() {
            if let GeometryEvaluationStatus::NeedsEvaluation = geometry.evaluation_status {
                geometry.evaluation_status = GeometryEvaluationStatus::Evaluating;
                let job = self.submit_evaluation_job(geometry_id, geometry);
                self.evaluation_jobs.push(job);
            }
        }
    }
    
    #[profiler::function]
    pub fn update_evaluated_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        
        let finished_indices: Vec<usize> = self.evaluation_jobs.iter_mut().enumerate()
            .filter_map(|(index, job)| {
                if job.join_handle.is_finished() { Some(index) } else { None }
            }).collect();
            
        for finished_index in finished_indices {
            profiler::scope!("Swap old SVO for new finished SVO");
            let job = self.evaluation_jobs.remove(finished_index);
            match job.join_handle.join() {
                Ok(svo) => {
                    if let Some(geometry) = geometry_pool.get_mut(job.geometry_id) {
                        info!("Finished evaluating geometry {:?}:", job.geometry_id);
                        geometry.svo = Some(svo);
                        geometry.evaluation_status = GeometryEvaluationStatus::Evaluated;
                    }
                },
                Err(error) => {
                    error!("Error while evaluating geometry {:?}: {:?}", job.geometry_id, error);
                    panic!("Error above was fatal, exiting...");
                }
            }
        }
    }
    
    #[profiler::function]
    fn submit_evaluation_job(&mut self, geometry_id: GeometryID, geometry: &mut Geometry) -> EvaluationJob {
        
        geometry.evaluation_status = GeometryEvaluationStatus::Evaluating;
        let edits = geometry.edits.clone();
        
        info!("Submitting geometry for evaluation job: {:?}", geometry_id);
        
        // Spawn a native evaluation thread and store its handle
        let gpu_resources = self.gpu_resources.clone();
        let join_handle = profiler::call!(
            std::thread::spawn(move || {
                // TODO: use some clever resource management to reuse allocated not used octree.
                Self::evaluate(
                    profiler::call!(svo::Octree::new(&gpu_resources.gpu, svo::Capacity::BrickPoolSide(12))),
                    edits,
                    gpu_resources,
                )
            })
        );
        
        EvaluationJob {
            join_handle,
            geometry_id,
        }
    }
    
}

// Internal evaluation algorithm
impl Evaluator {
    
    /// Function evaluating one edit list into an SVOctree
    /// The SVO exists in memory because it's allocated resources could be reused to store the new SVO.
    #[profiler::function]
    fn evaluate(mut svo: svo::Octree, edits: GeometryEditList, gpu_resources: EvaluationGPUResources) -> svo::Octree {
        
        let mut work_assignment_uniform = WorkAssignmentUniform::new(
            &gpu_resources.gpu,
            {
                // prepare the SVO for evaluation -> compute bounding cube
                let aabb = svo.aabb.get_or_insert_with(|| AABB::new(0.5 * glam::Vec3::NEG_ONE, 0.5 * glam::Vec3::ONE));
                // let aabb = svo.aabb.get_or_insert_with(|| edits.aabb); // TODO: use this when implemented
                
                // Get voxel size
                let min_voxel_size = 0.1; // NOTE: Arbitrary for now -> settable by gui into a property on geometry
                
                WorkAssignment::new(aabb.bounding_cube(), min_voxel_size)
            }
        );
        
        let mut dispatch_output_buffer = DispatchOutputBuffer::new(&gpu_resources.gpu);
        
        let execute_level = &mut |
            dispatch_output: DispatchOutput,
            dispatch_output_buffer: &mut DispatchOutputBuffer,
            work_assignment_uniform: &mut WorkAssignmentUniform
        | {
            profiler::scope!("Execute level");
            
            let mut encoder = profiler::call!(
                gpu_resources.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Evaluator Compute Encoder"),
                })
            );
            
            {
                let mut compute_pass = profiler::call!(
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Evaluator Compute Pass"),
                    })
                );
                
                compute_pass.insert_debug_marker("SVO Evaluation dispatch compute step");
                
                profiler::call!(compute_pass.set_pipeline(&gpu_resources.pipeline));
                {
                    profiler::scope!("Settings bind groups");
                    compute_pass.set_bind_group(0, &work_assignment_uniform.bind_group(&gpu_resources.gpu, &gpu_resources.work_assignment_layout), &[]);
                    compute_pass.set_bind_group(1, &svo.node_pool.bind_group(&gpu_resources.gpu, &gpu_resources.node_pool_bind_group_layout), &[]);
                    compute_pass.set_bind_group(2, &svo.brick_pool.bind_group(&gpu_resources.gpu, &gpu_resources.brick_pool_bind_group_layout), &[]);
                    compute_pass.set_bind_group(3, &dispatch_output_buffer.bind_group(&gpu_resources.gpu, &gpu_resources.dispatch_param_buffer_layout), &[]);
                }
                
                profiler::call!(
                    compute_pass.dispatch_workgroups(dispatch_output.unevaluated_nodes.max(1), 1, 1)
                );
                
            } // compute pass drops here
            
            profiler::call!(gpu_resources.gpu.queue.submit(Some(encoder.finish())));
            dispatch_output_buffer.read(&gpu_resources.gpu)
        };
        
        // root node
        let mut dispatch_output = execute_level(
            DispatchOutput { unevaluated_nodes: 0, start_index: 0 },
            &mut dispatch_output_buffer,
            &mut work_assignment_uniform,
        );
        
        // set uniform to not be root
        work_assignment_uniform.update(
            &gpu_resources.gpu,
            WorkAssignment { is_root: 0, ..work_assignment_uniform.work_assignment }
        );
        
        // each level
        loop {
            profiler::scope!("Dispatch loop");
            dispatch_output = execute_level(dispatch_output, &mut dispatch_output_buffer, &mut work_assignment_uniform);
            if dispatch_output.unevaluated_nodes == 0 {
                break;
            }
        }
        svo
    }
}

// ------------------------------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct WorkAssignment {
    // Bounding cube of the SVO evaluation domain. SVO will be fitted into this cube.
    svo_bounding_cube: BoundingCube,
    
    /// Minimum voxel size in world space - svo will be divided until voxel size is smaller than this value
    min_voxel_size: f32,
    
    /// If 1 then shader will evaluate as and only root brick creating first tile
    is_root: u32,
    
    /// padding
    _padding: [u32; 2],
}
impl WorkAssignment {
    pub fn new(svo_bounding_cube: BoundingCube, min_voxel_size: f32) -> Self {
        Self {
            svo_bounding_cube,
            min_voxel_size,
            is_root: 1,
            _padding: [0; 2],
        }
    }
}

struct WorkAssignmentUniform {
    
    /// Work assignment Data
    work_assignment: WorkAssignment,
    
    /// This structure represented in uniform buffer on GPU
    uniform_buffer: wgpu::Buffer,
    
    /// A bind group of this particular node pool.
    /// - When accessed through a `bind_group` method it will bew created.
    bind_group: Option<wgpu::BindGroup>,
}

// GPU binding
impl WorkAssignmentUniform {
    #[profiler::function]
    pub fn new(gpu: &GPUContext, work_assignment: WorkAssignment) -> Self {
        let v = [work_assignment.clone()];
        let a: &[u8] = bytemuck::cast_slice(&v);
        let uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Work Assignment Uniform Buffer"),
            contents: a,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
        });
        
        Self {
            work_assignment,
            uniform_buffer,
            bind_group: None,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, gpu: &GPUContext, work_assignment: WorkAssignment) {
        {
            let x = [work_assignment.clone()];
            let data: &[u8] = bytemuck::cast_slice(&x);
            let buffer_slice = self.uniform_buffer.slice(..);
            profiler::call!(buffer_slice.map_async(wgpu::MapMode::Write, move |_| ()));
            profiler::call!(gpu.device.poll(wgpu::Maintain::Wait));
            let mut mapped_data = profiler::call!(buffer_slice.get_mapped_range_mut());
            profiler::call!(mapped_data.clone_from_slice(data));
        }
        profiler::call!(self.uniform_buffer.unmap());
        self.work_assignment = work_assignment;
    }
    
    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn bind_group(&mut self, gpu: &GPUContext, layout: &wgpu::BindGroupLayout) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            self.bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("WorkAssignment Bind Group"),
                layout: layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                }],
            }));
        };
        self.bind_group.as_ref().unwrap()
    }
    
    /// Creates and returns a custom binding for the node pool.
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &GPUContext, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
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

// ------------------------------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DispatchOutput {
    unevaluated_nodes: u32,
    start_index: u32,
}

struct DispatchOutputBuffer {
    buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
}

impl DispatchOutputBuffer {
    
    #[profiler::function]
    pub fn new(gpu: &GPUContext) -> Self {
        let output = DispatchOutput {
            unevaluated_nodes: 0,
            start_index: 0,
        };
        
        let buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dispatch Params Buffer"),
            contents: bytemuck::cast_slice(&[output]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        });
        
        Self {
            buffer,
            bind_group: None,
        }
    }
    
    #[profiler::function]
    pub fn read(&self, gpu: &GPUContext) -> DispatchOutput {
        let buffer_slice = self.buffer.slice(..);
        profiler::call!(buffer_slice.map_async(wgpu::MapMode::Read, move |_| ()));
        profiler::call!(gpu.device.poll(wgpu::Maintain::Wait));
        let data = profiler::call!(buffer_slice.get_mapped_range().to_vec());
        let output = bytemuck::from_bytes::<DispatchOutput>(data.as_slice());
        profiler::call!(self.buffer.unmap());
        output.clone()
    }
    
    #[profiler::function]
    pub fn bind_group(&mut self, gpu: &GPUContext, layout: &wgpu::BindGroupLayout) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            self.bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Dispatch Output Bind Group"),
                layout: layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer.as_entire_binding(),
                }],
            }));
        };
        self.bind_group.as_ref().unwrap()
    }
    
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &GPUContext, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Dispatch Output Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }
    
}
