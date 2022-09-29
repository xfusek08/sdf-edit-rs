use dolly::prelude::{YawPitch, RightHanded};

use crate::app::{
    application::ControlFlowResultAction,
    updating::{UpdaterModule, UpdateContext, InputUpdateResult, ResizeContext}, camera::SmoothZoomArm,
};


#[derive(Default)]
pub struct CameraUpdater;

impl UpdaterModule for CameraUpdater {
    
    #[profiler::function]
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
        let camera = &mut context.state.scene.camera;
        
        let (dx, dy) = context.input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && context.input.mouse_held(0) {
            camera
                .rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
        }
        let scroll = context.input.scroll_diff();
        if scroll != 0.0 {
            camera
                .rig
                .driver_mut::<SmoothZoomArm<RightHanded>>()
                .scale_distance(1.0 + scroll * -0.3);
        }
        
        InputUpdateResult::default() // do not prevent event propagation
    }
    
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        let camera = &mut context.state.scene.camera;
        
        let orig = camera.rig.final_transform;
        let new = camera.rig.update(context.tick.delta.as_secs_f32());
        if orig.position != new.position || orig.rotation != new.rotation {
            return ControlFlowResultAction::Redraw;
        }
        ControlFlowResultAction::None
    }
    
    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction {
        context.state.scene.camera.aspect_ratio = context.size.width as f32 / context.size.height as f32;
        ControlFlowResultAction::None
    }
    
    fn after_render(&mut self, state: &mut crate::app::state::State) {}
    
}
