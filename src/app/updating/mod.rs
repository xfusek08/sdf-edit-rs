
mod camera_updater;
pub use camera_updater::CameraUpdater;

use winit::window::Window;
use winit_input_helper::WinitInputHelper;
use crate::app::{
    scene::Scene,
    clock::Tick, application::ControlFlowResultAction
};
use super::gui::Gui;

// Updater
// -------

pub struct Updater {
    modules: Vec<Box<dyn UpdaterModule>>,
    pub update_cnt: u64,
    pub input_cnt: u64,
}

impl Updater {
    pub fn new() -> Self {
        Self {
            modules: vec![],
            update_cnt: 0,
            input_cnt: 0,
        }
    }
    
    pub fn with_module<M>(mut self) -> Self
    where
        M: UpdaterModule + Default + 'static
    {
        self.modules.push(Box::new(M::default()));
        self
    }
    
    /// Invoked when input has changed
    #[profiler::function]
    pub fn input(&mut self, gui: &mut Gui, scene: &mut Scene, context: &UpdateContext) -> ControlFlowResultAction {
        let mut result = gui.update(scene, context);
        if result.handled {
            return result.result;
        }
        
        for module in self.modules.iter_mut() {
            result = result.combine(module.input(scene, context));
            if result.handled {
                break;
            }
        }
        self.input_cnt += 1;
        result.result
    }
    
    /// Invoked on tick
    #[profiler::function]
    pub fn update(&mut self, gui: &mut Gui, scene: &mut Scene, context: &UpdateContext) -> ControlFlowResultAction {
        let mut result = gui.update(scene, context).result;
        for module in self.modules.iter_mut() {
            result = result.combine(module.update(scene, context));
        }
        self.update_cnt += 1;
        result
    }
    
    /// React to resize event
    #[profiler::function]
    pub fn resize(&mut self, scene: &mut Scene, size: winit::dpi::PhysicalSize<u32>, scale_factor: f64) -> ControlFlowResultAction {
        let mut result = ControlFlowResultAction::None;
        for module in self.modules.iter_mut() {
            result = result.combine(module.resize(scene, size, scale_factor));
        }
        result
    }
}

// UpdaterModule
// -------------

pub trait UpdaterModule {
    fn input(&mut self, scene: &mut Scene, context: &UpdateContext) -> InputUpdateResult;
    fn update(&mut self, scene: &mut Scene, context: &UpdateContext) -> ControlFlowResultAction;
    fn resize(&mut self, scene: &mut Scene, size: winit::dpi::PhysicalSize<u32>, scale_factor: f64) -> ControlFlowResultAction;
}

pub struct UpdateContext<'a> {
    pub input: &'a WinitInputHelper,
    pub tick: &'a Tick,
    pub window: &'a Window
}

// InputUpdateResult
// -----------------

pub struct InputUpdateResult {
    pub handled: bool,
    pub result: ControlFlowResultAction
}
impl InputUpdateResult {
    pub fn combine(self, other: Self) -> Self {
        Self {
            handled: self.handled || other.handled,
            result: self.result.combine(other.result)
        }
    }
}
impl Default for InputUpdateResult {
    fn default() -> Self {
        Self {
            handled: false,
            result: ControlFlowResultAction::None
        }
    }
}
