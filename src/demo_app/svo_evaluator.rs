///! This is updater module which checks if there is a geometry requesting an SVO evaluation
///! And sends the svo of the geometry for evaluation
use std::sync::Arc;

use crate::demo_app::scene::Scene;
use crate::framework::{
    gpu,
    updater::{
        AfterRenderContext, InputUpdateResult, ResizeContext, UpdateContext, UpdateResultAction,
        UpdaterModule,
    },
};
use crate::sdf::evaluator::Evaluator;

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

impl UpdaterModule<Scene> for SvoEvaluatorUpdater {
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

    fn resize(&mut self, _: &mut ResizeContext<Scene>) -> UpdateResultAction {
        UpdateResultAction::None
    }

    fn after_render(&mut self, _: &mut AfterRenderContext<Scene>) {}
}
