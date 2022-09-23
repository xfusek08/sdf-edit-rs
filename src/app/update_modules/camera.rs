use dolly::prelude::{YawPitch, Arm};

use crate::app::{
    application::ControlFlowResultAction,
    updating::{UpdaterModule, UpdateContext, InputUpdateResult, ResizeContext},
};


#[derive(Default)]
pub struct CameraUpdater;

impl UpdaterModule for CameraUpdater {
    
    #[profiler::function]
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
        let (dx, dy) = context.input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && context.input.mouse_held(0) {
            context.scene.camera
                .rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
        }
        let scroll = context.input.scroll_diff();
        if scroll != 0.0 {
            context.scene.camera
                .rig
                .driver_mut::<Arm>()
                .offset *= 1.0 + scroll * -0.3;
        }
        
        InputUpdateResult::default() // do not prevent event propagation
    }
    
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        let orig = context.scene.camera.rig.final_transform;
        let new = context.scene.camera.rig.update(context.tick.delta.as_secs_f32());
        if orig.position != new.position || orig.rotation != new.rotation {
            return ControlFlowResultAction::Redraw;
        }
        ControlFlowResultAction::None
    }
    
    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction {
        context.scene.camera.aspect_ratio = context.size.width as f32 / context.size.height as f32;
        ControlFlowResultAction::None
    }
    
}
