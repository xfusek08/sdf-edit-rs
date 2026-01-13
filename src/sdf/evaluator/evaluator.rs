//! evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::sync::Arc;

use crate::{
    framework::gpu,
    sdf::{
        geometry::{self, EvaluationStatus, Geometry, GeometryPool},
        svo,
    },
};

use super::{EvaluationContext, KernelSVOLevel};
pub struct Evaluator {
    gpu: Arc<gpu::Context>, // this is and Arc because in the future evaluator will run on a separate thread asynchronously
    level_evaluation_kernel: KernelSVOLevel,
}

impl Evaluator {
    #[profiler::function]
    pub fn new(gpu: Arc<gpu::Context>) -> Self {
        let level_evaluation_kernel = KernelSVOLevel::new(&gpu);
        Self {
            gpu,
            level_evaluation_kernel,
        }
    }
}

impl Evaluator {
    #[profiler::function]
    pub fn evaluate_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        for (geometry_id, geometry) in geometry_pool.iter_mut() {
            if let EvaluationStatus::NeedsEvaluation = geometry.evaluation_status {
                geometry.evaluation_status = EvaluationStatus::Evaluating;
                self.evaluate_geometry(format!("{:?}", geometry_id), geometry);
            }
        }
    }

    #[profiler::function]
    pub fn update_evaluated_geometries(&mut self, geometry_pool: &mut GeometryPool) {
        // TODO: look for evaluation results in evaluator and update geometry when it has result for it ready
    }

    #[profiler::function(pinned)]
    fn evaluate_geometry(&mut self, svo_label: String, geometry: &mut Geometry) {
        // Get minimum voxel size for this evaluation run.
        let minium_voxel_size = geometry.min_voxel_size();

        // Compute Domain - An AABB in which the geometry is guaranteed to be fully contained
        let domain = geometry.total_aabb().bounding_cube();

        // TODO: Dispatch a kernel to compute the domain of the geometry (tightly fitted AABB)

        // Construct edit list in gpu memory
        let edits = geometry::GPUEdits::from_edit_list(&self.gpu, &geometry.edits());

        // Extract svo from geometry
        let svo = geometry
            .svo
            .take()
            .unwrap_or_else(|| svo::Svo::new(svo_label, &self.gpu, svo::Capacity::Nodes(100_000)));

        // Prepare level evaluation kernel for this run
        let level_kernel = &mut self.level_evaluation_kernel;
        level_kernel.set_context(
            EvaluationContext::new(&self.gpu, svo, edits),
            domain,
            minium_voxel_size,
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
        let EvaluationContext { mut svo, .. } = level_kernel
            .take_context()
            .expect("Fatal error: KernelSVOLevel did not return an svo");

        // Update SVO to reflect changes
        svo.levels = levels;
        svo.domain = domain;
        svo.node_pool.buffers_changed();
        svo.node_pool.load_count(&self.gpu);
        // svo.node_pool.trim_overflowing_levels(&self.gpu);

        // Store svo back in geometry
        geometry.svo = Some(svo);
        geometry.evaluation_status = EvaluationStatus::Evaluated;
    }
}
