use std::sync::Arc;

use crate::{
    framework::{
        gpu,
        math::AABB
    },
    sdf::{
        svo,
        geometry::GeometryEditList
    },
};

use super::{
    brick_padding_indices_uniform::BrickPaddingIndicesUniform,
    work_assignment::{
        WorkAssignmentUniform,
        WorkAssignment
    }
};

/// A struct containing all GPU resources and data needed to run evaluation algorithm
/// Note this could be merger into a evaluator pipeline object
#[derive(Clone)]
pub struct EvaluationGPUResources {
    pub gpu: Arc<gpu::Context>,
    pub pipeline: Arc<wgpu::ComputePipeline>,
    pub work_assignment_layout: Arc<wgpu::BindGroupLayout>,
    pub node_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub brick_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub brick_padding_indices_uniform: Arc<BrickPaddingIndicesUniform>,
}

/// Function evaluating one edit list into an SVOctree
/// The SVO exists in memory because it's allocated resources could be reused to store the new SVO.
#[profiler::function]
pub fn evaluate(mut svo: svo::Svo, edits: GeometryEditList, gpu_resources: EvaluationGPUResources, min_voxel_size: f32) -> svo::Svo {
    
    // 1. Work assignment uniform
    // a) prepare the SVO for evaluation -> compute bounding cube
    let aabb = svo.aabb.get_or_insert_with(|| AABB::new(0.5 * glam::Vec3::NEG_ONE, 0.5 * glam::Vec3::ONE));
    // TODO: when implemented: let aabb = svo.aabb.get_or_insert_with(|| edits.aabb);
    
    // c) Create work assignment uniform
    let mut work_assignment_uniform = WorkAssignmentUniform::new(
        &gpu_resources.gpu,
        WorkAssignment::new_root(aabb.bounding_cube(), min_voxel_size)
    );
    
    // 2. Lambda evaluating one SVO level (root if None given)
    let evaluate_level = &mut |
        level: Option<svo::Level> // None -> root
    | -> svo::Level {
        profiler::scope!("Evaluating a SVO level");
        
        dbg!("Evaluating level: {:?}", level);
        
        // Update start index uniform for dispatch
        let (
            start_index,
            node_count_to_evaluate,
            is_root
        ) = if let Some(svo::Level { start_index, node_count }) = level {
            (start_index, node_count, false)
        } else {
            (0, 1, true)
        };
        
        // set uniform to not be root
        work_assignment_uniform.update(&gpu_resources.gpu, WorkAssignment {
            is_root: if level.is_none() { 1 } else { 0 },
            start_index: start_index,
            ..work_assignment_uniform.work_assignment
        });
        
        let old_node_count = svo.node_pool.load_count(&gpu_resources.gpu);
        
        let uniform_bind_group = work_assignment_uniform.create_bind_group(&gpu_resources.gpu, &gpu_resources.work_assignment_layout);
        let node_bind_group = svo.node_pool.create_bind_group(&gpu_resources.gpu, &gpu_resources.node_pool_bind_group_layout);
        let brick_bind_group = svo.brick_pool.create_write_bind_group(&gpu_resources.gpu, &gpu_resources.brick_pool_bind_group_layout);
        
        // Command encoder for compute pass
        let mut encoder = profiler::call!(
            gpu_resources.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Evaluator Compute Pass Command Encoder"),
            })
        );
        
        {
            let mut compute_pass = profiler::call!(
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Evaluator Compute Pass"),
                })
            );
            
            compute_pass.insert_debug_marker("SVO Evaluation dispatch compute step");
            
            profiler::call!(
                compute_pass.set_pipeline(&gpu_resources.pipeline)
            );
            
            {
                profiler::scope!("Settings bind groups");
                compute_pass.set_bind_group(0, &uniform_bind_group, &[]);
                compute_pass.set_bind_group(1, &node_bind_group, &[]);
                compute_pass.set_bind_group(2, &brick_bind_group, &[]);
                compute_pass.set_bind_group(3, &gpu_resources.brick_padding_indices_uniform.bind_group, &[]);
            }
            
            profiler::call!(
                compute_pass.dispatch_workgroups(node_count_to_evaluate, 1, 1)
            );
            
        } // compute pass drops here
        
        profiler::call!(gpu_resources.gpu.queue.submit(Some(encoder.finish())));
        
        // Wait for queue to finish
        profiler::call!(gpu_resources.gpu.device.poll(wgpu::Maintain::Wait));
        
        // Read node count from buffer and calculate newly created level
        svo.node_pool.buffers_changed();
        let new_node_count = svo.node_pool.load_count(&gpu_resources.gpu);
        let added_nodes = new_node_count - old_node_count;
        
        if is_root {
            svo::Level {
                start_index: 0,
                node_count: added_nodes,
            }
        } else {
            svo::Level {
                start_index: start_index + node_count_to_evaluate,
                node_count: added_nodes,
            }
        }
    };
    
    // Root node
    let mut level = evaluate_level(None);
    svo.levels.push(level);
    
    // Evaluate levels until resulting level ha no more nodes to be evaluated
    loop { profiler::scope!("Dispatch loop");
        level = evaluate_level(Some(level));
        dbg!("Level: {:?}", level);
        if level.node_count == 0 {
            break; // end on first empty level
        } else {
            svo.levels.push(level); // register level into octree
        }
    }
    
    svo.node_pool.buffers_changed();
    svo.node_pool.load_count(&gpu_resources.gpu);
    svo
}
