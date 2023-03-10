use crate::framework::{
    application::Context,
    updater::Updater,
    gui::GuiUpdateModule,
    camera::CameraUpdater,
};

use super::{
    scene::Scene,
    modules::{
        TmpEvaluatorConfig,
        SvoEvaluatorUpdater,
    },
    gui_modules::{
        LegacyAppsGui,
        DynamicTestGeometry
    },
};

#[cfg(feature = "stats")]
use super::gui_modules::stats_gui::StatsGui;

#[cfg(feature = "counters")]
use super::gui_modules::counters_gui::CountersGui;

pub fn define_updater(context: &Context) -> Updater<Scene> {
    Updater::new()
        .with_module(GuiUpdateModule::new(vec![
            Box::new(LegacyAppsGui),
            Box::new(DynamicTestGeometry::new()),
            #[cfg(feature = "stats")]
            Box::new(StatsGui),
            #[cfg(feature = "counters")]
            Box::new(CountersGui),
        ]))
        .with_module(TmpEvaluatorConfig::default())
        .with_module(CameraUpdater)
        // .with_module(VoxelSizeReferenceDisplayer { visible: false })
        .with_module(SvoEvaluatorUpdater::new(context.gpu.clone())) // SVO updater needs arc reference to GPU context because it spawns threads sharing the GPU context
}
