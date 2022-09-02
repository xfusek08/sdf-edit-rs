use dolly::prelude::{YawPitch, Arm};
use winit_input_helper::WinitInputHelper;

use crate::app::{scene::Scene, clock::Tick};

pub enum UpdateResult {
    Wait, Redraw, Exit
}

pub struct Updater;

impl Updater {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Invoked when input has changed
    #[profiler::function]
    pub fn input(&mut self, mut scene: Scene, input: &WinitInputHelper, tick: &Tick) -> (UpdateResult, Scene) {
        let mut result = UpdateResult::Wait;
        let (dx, dy) = input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && input.mouse_held(0) {
            scene.camera
                .rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
            result = UpdateResult::Redraw;
        }
        let scroll = input.scroll_diff();
        if scroll != 0.0 {
            scene.camera
                .rig
                .driver_mut::<Arm>()
                .offset *= 1.0 + scroll * -0.3;
            result = UpdateResult::Redraw;
        }
        (result, scene)
    }
    
    /// Invoked on tick
    #[profiler::function]
    pub fn update(&mut self, mut scene: Scene, input: &WinitInputHelper, tick: &Tick) -> (UpdateResult, Scene) {
        let orig = scene.camera.rig.final_transform;
        let new = scene.camera.rig.update(tick.delta.as_secs_f32());
        if orig.position != new.position || orig.rotation != new.rotation {
            return (UpdateResult::Redraw, scene);
        }
        (UpdateResult::Wait, scene)
    }
}
