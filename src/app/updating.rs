

use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use super::clock::Tick;

use crate::app::{
    application::ControlFlowResultAction,
    state::State
};


// UpdateContext
// -------------

pub struct UpdateContext<'a> {
    pub state:  &'a mut State,
    pub input:  &'a WinitInputHelper,
    pub tick:   &'a Tick,
    pub window: &'a Window
}

pub struct ResizeContext<'a> {
    pub state:        &'a mut State,
    pub size:         &'a winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
}

// UpdaterModule
// -------------

pub trait UpdaterModule {
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult;
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction;
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction;
    fn after_render(&mut self, state: &mut State);
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
    
    pub fn with_module<M>(mut self, module: M) -> Self
    where
        M: UpdaterModule + 'static
    {
        self.modules.push(Box::new(module));
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
    
    #[profiler::function]
    pub fn after_render(&mut self, state: &mut State) {
        for module in self.modules.iter_mut() {
            module.after_render(state);
        }
    }
    
}
