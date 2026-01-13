use crate::framework::updater::{
    AfterRenderContext, InputUpdateResult, ResizeContext, UpdateContext, UpdateResultAction,
    UpdaterModule,
};

use super::{Camera, SceneWithCamera};

#[derive(Default)]
pub struct CameraUpdater;

impl<S: SceneWithCamera> UpdaterModule<S> for CameraUpdater {
    #[profiler::function]
    fn input(&mut self, context: &mut UpdateContext<S>) -> InputUpdateResult {
        context.scene.get_camera_mut().on_input(context.input);
        InputUpdateResult::default() // do not prevent event propagation
    }

    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<S>) -> UpdateResultAction {
        let camera_rig = context.scene.get_camera_mut();
        let orig_position = camera_rig.camera().position;
        let orig_rotation = camera_rig.camera().rotation;
        let t = camera_rig.update(context.tick.delta.as_secs_f32(), context.input);
        if orig_position != t.position || orig_rotation != t.rotation {
            return UpdateResultAction::Redraw;
        }
        UpdateResultAction::None
    }

    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext<S>) -> UpdateResultAction {
        let rig = context.scene.get_camera_mut();
        rig.set_camera(Camera {
            aspect_ratio: context.size.width as f32 / context.size.height as f32,
            ..*rig.camera()
        });
        UpdateResultAction::None
    }

    fn after_render(&mut self, _: &mut AfterRenderContext<S>) {}
}
