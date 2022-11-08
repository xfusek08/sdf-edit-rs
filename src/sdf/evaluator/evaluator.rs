//! evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{thread, sync::Arc, borrow::Cow};

use crate::{
    info,
    error,
    framework::gpu,
    sdf::{
        svo,
        geometry::{
            GeometryID,
            GeometryPool,
            GeometryEvaluationStatus,
            Geometry,
        },
    },
};

use super::{
    brick_padding_indices_uniform::BrickPaddingIndicesUniform,
    work_assignment::WorkAssignmentUniform,
    evaluate::{
        EvaluationGPUResources,
        evaluate
    },
};

struct EvaluationJob {
    join_handle: thread::JoinHandle<svo::Svo>,
    geometry_id: GeometryID,
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
    pub fn new(gpu: Arc<gpu::Context>) -> Evaluator {
        let work_assignment_layout = Arc::new(
            WorkAssignmentUniform::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let node_pool_bind_group_layout = Arc::new(
            svo::NodePool::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE, false)
        );
        let brick_pool_bind_group_layout = Arc::new(
            svo::BrickPool::create_write_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let brick_padding_indices_uniform = Arc::new(
            BrickPaddingIndicesUniform::new(gpu.as_ref())
        );
        
        let pipeline_layout = { profiler::scope!("Create evaluator pipeline layout");
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[
                    work_assignment_layout.as_ref(),                  // 0 - Work Assignment
                    node_pool_bind_group_layout.as_ref(),             // 1 - Node Pool
                    brick_pool_bind_group_layout.as_ref(),            // 2 - Brick Pool
                    &brick_padding_indices_uniform.bind_group_layout, // 3 - Brick Padding Indices
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
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("_shader.wgsl"))),
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
                brick_padding_indices_uniform,
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
                        geometry.svo = Some(Arc::new(svo));
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
        let min_voxel_size = geometry.min_voxel_size();
        
        info!("Submitting geometry for evaluation job: {:?}", geometry_id);
        
        // Spawn a native evaluation thread and store its handle
        let gpu_resources = self.gpu_resources.clone();
        let join_handle = profiler::call!(
            std::thread::spawn(move || {
                // TODO: use some clever resource management to reuse allocated not used octree.
                evaluate(
                    svo::Svo::new(&gpu_resources.gpu, svo::Capacity::BrickPoolSide(20)),
                    edits,
                    gpu_resources,
                    min_voxel_size
                )
            })
        );
        
        EvaluationJob {
            join_handle,
            geometry_id,
        }
    }
    
}
