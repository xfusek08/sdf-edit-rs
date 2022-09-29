///! This is updater module which checks if there is a geometry requesting an SVO evaluation
///! And sends the svo of the geometry for evaluation

use std::sync::Arc;

use crate::app::{
    application::ControlFlowResultAction,
    gpu::GPUContext,
    updating::{UpdaterModule, UpdateContext, InputUpdateResult},
    sdf::evaluator::Evaluator,
};

pub struct SVOUpdater {
    evaluator: Evaluator,
}

impl SVOUpdater {
    pub fn new(gpu: Arc<GPUContext>) -> SVOUpdater {
        Self {
            evaluator: Evaluator::new(gpu),
        }
    }
}

impl<'a> UpdaterModule for SVOUpdater {
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        let geometry_pool = &mut context.state.scene.geometry_pool;
        
        self.evaluator.evaluate_geometries(geometry_pool);
        self.evaluator.update_evaluated_geometries(geometry_pool);
        ControlFlowResultAction::None
    }

    fn input(&mut self, _: &mut UpdateContext) -> crate::app::updating::InputUpdateResult {
        InputUpdateResult::default()
    }

    fn resize(&mut self, _: &mut crate::app::updating::ResizeContext) -> ControlFlowResultAction {
        ControlFlowResultAction::None
    }
    
    fn after_render(&mut self, state: &mut crate::app::state::State) {}
}
