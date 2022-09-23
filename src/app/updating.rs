

use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::app::{gui::Gui, application::ControlFlowResultAction, scene::Scene};

use super::clock::Tick;

// UpdateContext
// -------------

pub struct UpdateContext<'a> {
    pub gui:    &'a mut Gui,
    pub scene:  &'a mut Scene,
    pub input:  &'a WinitInputHelper,
    pub tick:   &'a Tick,
    pub window: &'a Window
}

pub struct ResizeContext<'a> {
    pub scene:        &'a mut Scene,
    pub size:         &'a winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
}

// UpdaterModule
// -------------

pub trait UpdaterModule {
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult;
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction;
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction;
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
    pub fn input(&mut self, mut context: UpdateContext) -> ControlFlowResultAction {
        let mut result = InputUpdateResult::default();
        for module in self.modules.iter_mut() {
            result = result.combine(module.input(&mut context));
            if result.handled {
                break;
            }
        }
        self.input_cnt += 1;
        result.result
    }
    
    /// Invoked on tick
    #[profiler::function]
    pub fn update(&mut self, mut context: UpdateContext) -> ControlFlowResultAction {
        let mut result = ControlFlowResultAction::None;
        for module in self.modules.iter_mut() {
            result = result.combine(module.update(&mut context));
        }
        self.update_cnt += 1;
        result
    }
    
    /// React to resize event
    #[profiler::function]
    pub fn resize(&mut self, mut context: ResizeContext) -> ControlFlowResultAction {
        let mut result = ControlFlowResultAction::None;
        for module in self.modules.iter_mut() {
            result = result.combine(module.resize(&mut context));
        }
        result
    }
    
}
