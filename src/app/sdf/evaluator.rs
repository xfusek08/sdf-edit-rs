
// evaluator is meant to run asynchronously, and is responsible for computing a geometry octree from its edit list

use std::{thread, sync::Arc};

use crate::{app::gpu::GPUContext, info};

use super::{
    geometry::{Geometry, GeometryID, GeometryEditList, GeometryEvaluationStatus, GeometryPool},
    svo::{SVOctree, SVOctreeCapacity}
};

pub struct EvaluationJob {
    join_handle: thread::JoinHandle<SVOctree>,
    geometry_id: GeometryID,
}

pub struct Evaluator {
    gpu: Arc<GPUContext>,
    evaluation_jobs: Vec<EvaluationJob>,
}

impl Evaluator {
    
    pub fn new(gpu: Arc<GPUContext>) -> Evaluator {
        Self {
            gpu,
            evaluation_jobs: vec![],
        }
    }
    
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
            let job = self.evaluation_jobs.remove(finished_index);
            let svo = job.join_handle.join().unwrap();
            if let Some(geometry) = geometry_pool.get_mut(job.geometry_id) {
                dbg!("Finished evaluating geometry:");
                dbg!(job.geometry_id);
                geometry.svo = Some(svo);
                geometry.evaluation_status = GeometryEvaluationStatus::Evaluated;
            }
        }
    }
    
    #[profiler::function]
    fn submit_evaluation_job(&mut self, geometry_id: GeometryID, geometry: &mut Geometry) -> EvaluationJob {
        
        geometry.evaluation_status = GeometryEvaluationStatus::Evaluating;
        let edits = geometry.edits.clone();
        let gpu = self.gpu.clone();
        
        info!("Evaluation geometry:");
        dbg!(geometry_id);
        
        // Spawn a native evaluation thread and store its handle
        let join_handle = std::thread::spawn(move || {
            
            // create a target octree
            let svo = profiler::call!(
                SVOctree::new(SVOctreeCapacity::Nodes(1000), gpu.as_ref())
            );
            
            Self::evaluate(gpu.as_ref(), svo, edits)
        });
        
        EvaluationJob {
            join_handle,
            geometry_id,
        }
    }
    
    /// Function evaluating one edit list into an SVOctree
    /// The SVO exists in memory because it's allocated resources could be reused to store the new SVO.
    #[profiler::function]
    fn evaluate(gpu: &GPUContext, svo: SVOctree, edits: GeometryEditList) -> SVOctree {
        // sleep for 1 second to simulate a long evaluation
        dbg!("Evaluating geometry");
        thread::sleep(std::time::Duration::from_secs(1));
        return svo;
    }
    
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
