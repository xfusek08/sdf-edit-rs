
// evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{sync::Arc, time::Duration};

use crate::app::gpu::GPUContext;

use super::{geometry::Geometry, svo::SVOctree};

pub struct Evaluator {
    evaluation_jobs: Vec<EvaluationJob>,
}

impl Evaluator {
    
    /// Spawns an asynchronous evaluation job
    pub fn start_valuation(&mut self, geometry: Arc<Geometry>, gpu_context: &GPUContext) {
        
        // Take ownership of geometry octree to be evaluated
        let octree = geometry.svo.take();
        
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(3));
            octree
        });
        
        self.evaluation_jobs.push(EvaluationJob {
            thread_handle: handle,
            geometry:      geometry,
        });
    }
    
    /// Checks all registered evaluation jobs and for each completed job, updates the geometry octree
    pub fn check_finished_jobs(&self) {
        
    }
}

struct EvaluationJob {
    thread_handle: std::thread::JoinHandle<Option<SVOctree>>,
    geometry: Arc<Geometry>,
}
