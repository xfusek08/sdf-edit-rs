//! evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{sync::Arc, borrow::Cow};

use crate::{
    framework::gpu,
    sdf::{
        svo,
        geometry::{
            self,
            GeometryPool,
            GeometryEvaluationStatus,
            Geometry,
            GeometryEditsGPU,
        },
    },
};

use super::{
    brick_padding_indices_uniform::BrickPaddingIndicesUniform,
    dispatch_assignment::DispatchAssignmentUniform,
    evaluate::{
        EvaluationGPUResources,
        evaluate
    },
};

pub struct Evaluator {
    gpu_resources: EvaluationGPUResources,
}

// Construction
impl Evaluator {
    
    #[profiler::function]
    pub fn new(gpu: Arc<gpu::Context>) -> Evaluator {
        let dispatch_assignment_layout = Arc::new(
            DispatchAssignmentUniform::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let node_pool_bind_group_layout = Arc::new(
            svo::NodePool::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE, false)
        );
        let brick_pool_bind_group_layout = Arc::new(
            svo::BrickPool::create_write_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let geometry_edits_bing_group_layout = Arc::new(
            GeometryEditsGPU::create_bind_group_layout(gpu.as_ref(), wgpu::ShaderStages::COMPUTE)
        );
        let brick_padding_indices_uniform = Arc::new(
            BrickPaddingIndicesUniform::new(gpu.as_ref())
        );
        
        let pipeline_layout = { profiler::scope!("Create evaluator pipeline layout");
            gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[
                    dispatch_assignment_layout.as_ref(),                  // 0 - Work Assignment
                    node_pool_bind_group_layout.as_ref(),             // 1 - Node Pool
                    brick_pool_bind_group_layout.as_ref(),            // 2 - Brick Pool
                    geometry_edits_bing_group_layout.as_ref(),        // 3 - Geometry Edits
                    &brick_padding_indices_uniform.bind_group_layout, // 4 - Brick Padding Indices
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
            gpu_resources: EvaluationGPUResources {
                gpu,
                pipeline,
                dispatch_assignment_layout,
                node_pool_bind_group_layout,
                brick_pool_bind_group_layout,
                geometry_edits_bing_group_layout,
                brick_padding_indices_uniform,
            },
        }
    }
}

// Geometry evaluation job management (public interface)
impl Evaluator {
    
    #[profiler::function]
    pub fn evaluate_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        for (_, geometry) in geometry_pool.iter_mut() {
            if let GeometryEvaluationStatus::NeedsEvaluation = geometry.evaluation_status {
                geometry.evaluation_status = GeometryEvaluationStatus::Evaluating;
                self.evaluate_geometry(geometry);
            }
        }
    }
    
    #[profiler::function]
    pub fn update_evaluated_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        // TODO: look for evaluation results in evaluator and update geometry when it has result for it ready
    }
    
    #[profiler::function]
    fn evaluate_geometry(&mut self, geometry: &mut Geometry) {

        // TODO: create a evaluator object which handles its own input and output queue and single thread which evaluates the queue.
        
        let gpu_resources = self.gpu_resources.clone();
        let geometry_resources = match geometry.gpu_resources.take() {
            Some(resources) => resources,
            None => {
                let edits = GeometryEditsGPU::from_edit_list(&gpu_resources.gpu, &geometry.edits);
                let svo = svo::Svo::new(&gpu_resources.gpu, svo::Capacity::Depth(5));
                geometry::GPUResources { edits, svo }
            }
        };
        geometry.gpu_resources = Some(evaluate(gpu_resources, geometry_resources, geometry.min_voxel_size()));
        geometry.evaluation_status = GeometryEvaluationStatus::Evaluated;
    }
    
}
