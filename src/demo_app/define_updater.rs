use crate::framework::{
    application::Context,
    updater::Updater, gui::GuiUpdateModule, camera::CameraUpdater,
};

use super::{
    scene::Scene,
    draw_gui,
    modules::{
        tmp_evaluator_config::TmpEvaluatorConfig,
        voxel_size_reference_displayer::VoxelSizeReferenceDisplayer,
        svo_evaluator::SvoEvaluatorUpdater
    },
};


pub fn define_updater(context: &Context) -> Updater<Scene> {
    Updater::new()
        .with_module(GuiUpdateModule::new(draw_gui))
        .with_module(TmpEvaluatorConfig::default())
        .with_module(CameraUpdater)
        .with_module(VoxelSizeReferenceDisplayer { visible: false })
        .with_module(SvoEvaluatorUpdater::new(context.gpu.clone())) // SVO updater needs arc reference to GPU context because it spawns threads sharing the GPU context
}
