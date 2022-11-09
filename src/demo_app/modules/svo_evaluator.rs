///! This is updater module which checks if there is a geometry requesting an SVO evaluation
///! And sends the svo of the geometry for evaluation

use std::sync::Arc;

use crate::demo_app::scene::Scene;
use crate::sdf::evaluator::Evaluator;
use crate::framework::{
    gpu,
    updater::{
        UpdateContext,
        UpdateResultAction,
        InputUpdateResult,
        ResizeContext,
        AfterRenderContext,
        UpdaterModule
    },
};

pub struct SvoEvaluatorUpdater {
    evaluator: Evaluator,
}

impl SvoEvaluatorUpdater {
    pub fn new(gpu: Arc<gpu::Context>) -> SvoEvaluatorUpdater {
        Self {
            evaluator: Evaluator::new(gpu),
        }
    }
}

impl<'a> UpdaterModule<Scene> for SvoEvaluatorUpdater {
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<Scene>) -> UpdateResultAction {
        let geometry_pool = &mut context.scene.geometry_pool;
        
        self.evaluator.evaluate_geometries(geometry_pool);
        self.evaluator.update_evaluated_geometries(geometry_pool);
        UpdateResultAction::None
    }

    fn input(&mut self, _: &mut UpdateContext<Scene>) -> InputUpdateResult {
        InputUpdateResult::default()
    }

    fn resize(&mut self, context: &mut ResizeContext<Scene>) -> UpdateResultAction {
        UpdateResultAction::None
    }

    fn after_render(&mut self, state: &mut AfterRenderContext<Scene>) {}
}
