//! evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::sync::Arc;

use lazy_static::__Deref;

use crate::{
    framework::{gpu, math::AABB},
    sdf::{
        svo,
        geometry::{
            self,
            GeometryPool,
            EvaluationStatus,
            Geometry,
        },
    },
};

use super::{KernelSVOLevel, EvaluationContext};

pub struct Evaluator {
    gpu: Arc<gpu::Context>, // this is and Arc because in the future evaluator will run on a separate thread asynchronously
    level_evaluation_kernel: KernelSVOLevel,
}

impl Evaluator {
    #[profiler::function]
    pub fn new(gpu: Arc<gpu::Context>) -> Self {
        let level_evaluation_kernel = KernelSVOLevel::new(gpu.deref());
        Self { gpu, level_evaluation_kernel }
    }
}

impl Evaluator {
    
    #[profiler::function]
    pub fn evaluate_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        for (_, geometry) in geometry_pool.iter_mut() {
            if let EvaluationStatus::NeedsEvaluation = geometry.evaluation_status {
                geometry.evaluation_status = EvaluationStatus::Evaluating;
                self.evaluate_geometry(geometry);
            }
        }
    }
    
    #[profiler::function]
    pub fn update_evaluated_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        // TODO: look for evaluation results in evaluator and update geometry when it has result for it ready
    }
    
    #[profiler::function(pinned)]
    fn evaluate_geometry(&mut self, geometry: &mut Geometry) {
        
        // Get minimum voxel size for this evaluation run.
        let minium_voxel_size = geometry.min_voxel_size();
        
        // Compute Domain - An AABB in which the geometry is guaranteed to be fully contained
        let domain = geometry.total_aabb().bounding_cube();
        
        // Construct edit list in gpu memory
        let edits = geometry::GPUEdits::from_edit_list(&self.gpu, &geometry.edits);
        
        // Extract svo from geometry
        let svo = geometry.svo.take().unwrap_or_else(|| {
            svo::Svo::new(&self.gpu, svo::Capacity::Depth(5))
        });
        
        // Prepare leven evaluation kernel for this run
        let level_kernel = &mut self.level_evaluation_kernel;
        level_kernel.set_context(
            EvaluationContext::new(&self.gpu, svo, edits),
            domain,
            minium_voxel_size
        );
        
        // Clean svo level list - we will rebuild it
        let mut levels: Vec<svo::Level> = Vec::with_capacity(5);
        
        // Evaluation algorithm - evaluate levels in top-down manner until no more nodes are created
        let mut level = level_kernel.evaluate_root(&self.gpu);
        loop {
            // Register level into octree
            levels.push(level);
            
            // Evaluate next level
            level = level_kernel.evaluate_level(&self.gpu, &level);
            
            // If returned level is empty - no mo nodes were created so it is not a valid level and evaluation is done
            if level.node_count == 0 {
                break;
            }
        }
        
        // Retrieve svo from kernel
        let EvaluationContext  { mut svo, .. } = level_kernel.take_context().expect("Fatal error: KernelSVOLevel did not return an svo");
        
        // Update SVO to reflect changes
        svo.levels = levels;
        svo.domain = domain;
        svo.node_pool.buffers_changed();
        svo.node_pool.load_count(&self.gpu);
        
        // Store svo back in geometry
        geometry.svo = Some(svo);
        geometry.evaluation_status = EvaluationStatus::Evaluated;
    }
    
}
