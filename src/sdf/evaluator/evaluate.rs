use std::sync::Arc;

use crate::{
    sdf::{ svo, geometry },
    framework::{
        gpu,
        math::AABB
    },
};

use super::{
    brick_padding_indices_uniform::BrickPaddingIndicesUniform,
    dispatch_assignment::{
        DispatchAssignmentUniform,
        DispatchAssignment
    }
};

/// A struct containing all GPU resources and data needed to run evaluation algorithm
/// Note this could be merger into a evaluator pipeline object
#[derive(Clone)]
pub struct EvaluationGPUResources {
    pub gpu: Arc<gpu::Context>,
    pub pipeline: Arc<wgpu::ComputePipeline>,
    pub dispatch_assignment_layout: Arc<wgpu::BindGroupLayout>,
    pub node_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub brick_pool_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub geometry_edits_bing_group_layout: Arc<wgpu::BindGroupLayout>,
    pub brick_padding_indices_uniform: Arc<BrickPaddingIndicesUniform>,
}

/// Function evaluating one edit list into an SVOctree
/// The SVO exists in memory because it's allocated resources could be reused to store the new SVO.
#[profiler::function]
pub fn evaluate(
    evaluator_gpu_resources: EvaluationGPUResources,
    geometry_gpu_resources: geometry::GPUResources, // this evaluator run is sole owner of this data
    min_voxel_size: f32, // TODO: work assignment?
) -> geometry::GPUResources {
    
    let geometry::GPUResources {
        edits,
        mut svo,
    } = geometry_gpu_resources;
    
    let EvaluationGPUResources {
        gpu,
        pipeline,
        dispatch_assignment_layout,
        node_pool_bind_group_layout,
        brick_pool_bind_group_layout,
        geometry_edits_bing_group_layout,
        brick_padding_indices_uniform,
    } = evaluator_gpu_resources;
    
    
    // Create work assignment uniform
    //    - Will prepare work domain for root level
    //    - Will be updated for each level
    // --------------------------------------------
    
    let aabb = AABB::new(0.5 * glam::Vec3::NEG_ONE, 0.5 * glam::Vec3::ONE);
    let mut dispatch_assignment_uniform = DispatchAssignmentUniform::new(
        &gpu,
        DispatchAssignment::new_root(aabb.bounding_cube(), min_voxel_size)
    );
    
    
    // Prepare SVO
    //  - It is possible that the SVO has been used before and contains data, thus levels need to be cleared
    //  - when launching kernel for root level, node count will be zeroed so we do not need to update it here (from CPU)
    // -----------------------------------------------------------------------------------------------------------------
    svo.levels.clear();
    
    
    // Create bind groups on GPU
    // -------------------------
    
    let uniform_bind_group = dispatch_assignment_uniform.create_bind_group(&gpu, &dispatch_assignment_layout);
    let node_bind_group = svo.node_pool.create_bind_group(&gpu, &node_pool_bind_group_layout);
    let geometry_edits_bind_group = edits.create_bind_group(&gpu, &geometry_edits_bing_group_layout);
    let brick_bind_group = svo.brick_pool.create_write_bind_group(&gpu, &brick_pool_bind_group_layout);
    
    
    // Lambda evaluating one SVO level (root if None given)
    //   - Returns the next level to evaluate
    // ----------------------------------------------------
    
    let evaluate_level = &mut | level: Option<svo::Level> | -> svo::Level {
        profiler::scope!("Evaluating a SVO level", pinned);
        
        // Prepare variables for this level dependent on whether it is root or not
        let (
            start_index,
            node_count_to_evaluate,
            old_node_count,
            is_root,
        ) = match level {
            
            // non-root level
            Some(svo::Level { start_index, node_count }) => (
                start_index,
                node_count,
                svo.node_pool.load_count(&gpu), // Copy node count from GPU (probably not has been updated in previous level)
                false
            ),
            
            // root level
            None => (0, 1, 0, true),
        };
        
        // Upload dispatch assignment uniform to GPU
        dispatch_assignment_uniform.update(&gpu, DispatchAssignment {
            is_root: if level.is_none() { 1 } else { 0 },
            start_index: start_index,
            ..dispatch_assignment_uniform.work_assignment
        });
        
        // Create command encoder
        let mut encoder = {
            profiler::scope!("Creating command encoder");
            gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Evaluator Compute Pass Command Encoder"),
            })
        };
        
        // Create compute pass
        let mut compute_pass = {
            profiler::scope!("Creating Compute Pass");
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Evaluator Compute Pass"),
            })
        };
        
        { profiler::scope!("Setting pipeline");
            compute_pass.set_pipeline(&pipeline);
        }
        
        { profiler::scope!("Settings bind groups");
            compute_pass.set_bind_group(0, &uniform_bind_group, &[]);
            compute_pass.set_bind_group(1, &node_bind_group, &[]);
            compute_pass.set_bind_group(2, &brick_bind_group, &[]);
            compute_pass.set_bind_group(3, &geometry_edits_bind_group, &[]);
            compute_pass.set_bind_group(4, &brick_padding_indices_uniform.bind_group, &[]);
        }
        
        { profiler::scope!("Dispatch");
            compute_pass.dispatch_workgroups(node_count_to_evaluate, 1, 1);
        }
        
        { profiler::scope!("Drop Compute Pass");
           drop(compute_pass);
        }
        
        { profiler::scope!("Submit command encoder to queue");
            gpu.queue.submit(Some(encoder.finish()));
        }
        
        { profiler::scope!("Wait for queue to finish computation");
            profiler::call!(gpu.device.poll(wgpu::Maintain::Wait));
        }
        
        // Read node count from buffer and calculate newly created level
        svo.node_pool.buffers_changed();
        let current_node_count = svo.node_pool.load_count(&gpu);
        
        // Return next unevaluated level
        let node_count = current_node_count - old_node_count;
        if is_root {
            svo::Level { start_index: 0, node_count }
        } else {
            svo::Level { start_index: start_index + node_count_to_evaluate, node_count }
        }
        
    }; // End of evaluate lambda
    
    
    // Evaluation algorithm using the lambda
    // -------------------------------------
    
    // a) Create root level
    let mut level = evaluate_level(None);
    svo.levels.push(level);
    
    // b) Create child levels until no more nodes are created
    loop {
        // Evaluate next level
        level = evaluate_level(Some(level));
        // End on first empty level
        if level.node_count == 0 { break; }
        // Register level into octree
        svo.levels.push(level);
    }
    
    // Finalize and return
    // -------------------
    
    svo.node_pool.buffers_changed();
    svo.node_pool.load_count(&gpu);
    
    return geometry::GPUResources { svo, edits };
}
