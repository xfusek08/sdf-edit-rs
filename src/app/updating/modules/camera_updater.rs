use dolly::prelude::{YawPitch, Arm};

use crate::app::{
    updating::{UpdaterModule, InputUpdateResult, UpdateContext},
    scene::Scene,
    application::ControlFlowResultAction,
};


#[derive(Default)]
pub struct CameraUpdater;

impl UpdaterModule for CameraUpdater {
    #[profiler::function]
    fn input(&mut self, scene: &mut Scene, context: &UpdateContext) -> InputUpdateResult {
        let mut result = InputUpdateResult::default();
        let (dx, dy) = context.input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && context.input.mouse_held(0) {
            scene.camera
                .rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
            result.result = ControlFlowResultAction::Redraw;
        }
        let scroll = context.input.scroll_diff();
        if scroll != 0.0 {
            scene.camera
                .rig
                .driver_mut::<Arm>()
                .offset *= 1.0 + scroll * -0.3;
            result.result = ControlFlowResultAction::Redraw;
        }
        result
    }
    
    #[profiler::function]
    fn update(&mut self, scene: &mut Scene, context: &UpdateContext) -> ControlFlowResultAction {
        let orig = scene.camera.rig.final_transform;
        let new = scene.camera.rig.update(context.tick.delta.as_secs_f32());
        if orig.position != new.position || orig.rotation != new.rotation {
            return ControlFlowResultAction::Redraw;
        }
        ControlFlowResultAction::None
    }
}
