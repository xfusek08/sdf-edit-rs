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
    pub fn input(&mut self, scene: Scene, input: &WinitInputHelper, tick: &Tick) -> (UpdateResult, Scene) {
        (UpdateResult::Wait, scene)
    }
    
    
    /// Invoked on tick
    pub fn update(&mut self, scene: Scene, input: &WinitInputHelper, tick: &Tick) -> (UpdateResult, Scene) {
        (UpdateResult::Wait, scene)
    }
}
