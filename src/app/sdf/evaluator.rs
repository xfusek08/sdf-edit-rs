
// evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{thread, sync::Arc, borrow::Cow};

use wgpu::util::DeviceExt;

use crate::{app::gpu::GPUContext, info, error};

use super::{
    svo,
    bounding_volumes::BoundingCube,
    geometry::{
        Geometry,
        GeometryID,
        GeometryPool,
        GeometryEvaluationStatus, GeometryEditList,
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
    pub job_buffer_bind_group_layout: Arc<wgpu::BindGroupLayout>,
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
            WorkAssignmentResource::create_write_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let node_pool_bind_group_layout = Arc::new(
            svo::NodePool::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let brick_pool_bind_group_layout = Arc::new(
            svo::BrickPool::create_write_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let job_buffer_bind_group_layout = Arc::new(
            JobBuffer::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        
        let pipeline_layout = { profiler::scope!("Create evaluator pipeline layout");
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[
                    work_assignment_layout.as_ref(),      // 0 - Work Assignment
                    node_pool_bind_group_layout.as_ref(),  // 1 - Node Pool
                    brick_pool_bind_group_layout.as_ref(), // 2 - Brick Pool
                    job_buffer_bind_group_layout.as_ref(), // 3 - Job buffer
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
                job_buffer_bind_group_layout,
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
        
        
        dbg!(&geometry_id);
        
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
    // NOTE: Masks are not meant to be used on CPU side - this is only for debugging purposes such as reading (parsing) the contents of the buffers for debug display.
    const OCTREE_SUBDIVIDE_THIS_BIT: u32 = 0b10000000_00000000_00000000_00000000;
    const OCTREE_HAS_BRICK_BIT:      u32 = 0b01000000_00000000_00000000_00000000;
    const OCTREE_NODE_FLAGS_MASK:    u32 = 0b11000000_00000000_00000000_00000000;
    const OCTREE_CHILD_POINTER_MASK: u32 = 0b00111111_11111111_11111111_11111111;
    
    /// Function evaluating one edit list into an SVOctree
    /// The SVO exists in memory because it's allocated resources could be reused to store the new SVO.
    #[profiler::function]
    fn evaluate(svo: svo::Octree, edits: GeometryEditList, gpu_resources: EvaluationGPUResources) -> svo::Octree {
        // As a tmp solution, we just return a default SVO after 1 second
        info!("evaluate");
        thread::sleep(std::time::Duration::from_millis(500));
        info!("evaluate 2");
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
}

struct WorkAssignmentResource {
    
    /// Work assignment Data
    work_assignment: WorkAssignment,
    
    /// This structure represented in uniform buffer on GPU
    uniform_buffer: wgpu::Buffer,
    
    /// A bind group of this particular node pool.
    /// - When accessed through a `bind_group` method it will bew created.
    bind_group: Option<wgpu::BindGroup>,
}

// GPU binding
impl WorkAssignmentResource {
    #[profiler::function]
    pub fn new(gpu: &GPUContext, work_assignment: WorkAssignment) -> Self {
        let uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[work_assignment]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        Self {
            work_assignment,
            uniform_buffer,
            bind_group: None,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, gpu: &GPUContext, work_assignment: WorkAssignment) {
        gpu.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[work_assignment]),
        );
        self.work_assignment = work_assignment;
    }
    
    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn bind_group(&mut self, gpu: &GPUContext, layout: &wgpu::BindGroupLayout) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            self.bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SVO Node Pool Bind Group"),
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
    pub fn create_write_bind_group_layout(gpu: &GPUContext, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SVO Node Pool Bind Group Layout"),
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
struct JobBufferMeta {
    active_jobs: u32,
    job_count: u32,
    job_capacity: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Job {
    status: u32,
    node_index: u32,
}

struct JobBuffer {
    meta: JobBufferMeta,
    job_meta_buffer: wgpu::Buffer,
    job_buffer: wgpu::Buffer,
}

impl JobBuffer {
    
    #[profiler::function]
    pub fn new(gpu: &GPUContext, job_capacity: u32) -> Self {
        let meta = JobBufferMeta {
            active_jobs: 0,
            job_count: 0,
            job_capacity,
        };
        let job_meta_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[meta]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let job_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Job>() as u64 * job_capacity as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            meta,
            job_meta_buffer,
            job_buffer,
        }
    }
    
    #[profiler::function]
    pub fn update(&mut self, gpu: &GPUContext, jobs: &[Job]) {
        self.meta.job_count = jobs.len() as u32;
        profiler::call!(
            gpu.queue.write_buffer(&self.job_meta_buffer, 0, bytemuck::cast_slice(&[self.meta]))
        );
        profiler::call!(
            gpu.queue.write_buffer(&self.job_buffer, 0, bytemuck::cast_slice(jobs))
        );
    }
    
    #[profiler::function]
    pub fn bind_group(&self, gpu: &GPUContext, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Job Buffer Bind Group"),
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.job_meta_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.job_buffer.as_entire_binding(),
                },
            ],
        })
    }
    
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &GPUContext, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Job Buffer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        })
    }
    
}
