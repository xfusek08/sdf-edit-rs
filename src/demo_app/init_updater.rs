use crate::framework::{
    application::Context,
    updater::Updater,
    gui::GuiUpdateModule,
    camera::CameraUpdater,
};

use super::{
    scene::Scene,
    tmp_evaluator_config::TmpEvaluatorConfig,
    svo_evaluator::SvoEvaluatorUpdater,
    gui_modules::{
        LegacyAppsGui,
        DynamicTestGeometry, CameraGuiModule
    },
};

#[cfg(feature = "stats")]
use super::gui_modules::stats_gui::StatsGui;

#[cfg(feature = "counters")]
use super::gui_modules::counters_gui::CountersGui;

pub fn init_updater(context: &Context) -> Updater<Scene> {
    Updater::new()
        .with_module(GuiUpdateModule::new(vec![
            #[cfg(feature = "counters")]
            Box::new(CountersGui),
            Box::new(CameraGuiModule),
            Box::new(LegacyAppsGui),
            Box::new(DynamicTestGeometry::new()),
            #[cfg(feature = "stats")]
            Box::new(StatsGui),
        ]))
        .with_module(TmpEvaluatorConfig::default())
        .with_module(CameraUpdater)
        // .with_module(VoxelSizeReferenceDisplayer { visible: false })
        .with_module(SvoEvaluatorUpdater::new(context.gpu.clone())) // SVO updater needs arc reference to GPU context because it spawns threads sharing the GPU context
}
