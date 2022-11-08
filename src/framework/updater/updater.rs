
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::{framework::{clock::Tick, gui::Gui}};

// Contexts
// -------------

pub struct UpdateContext<'a, Scene> {
    pub gui:    &'a mut Gui,
    pub scene:  &'a mut Scene,
    pub input:  &'a WinitInputHelper,
    pub tick:   &'a Tick,
    pub window: &'a Window
}

pub struct ResizeContext<'a, Scene> {
    pub gui:          &'a mut Gui,
    pub scene:        &'a mut Scene,
    pub size:         &'a winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
}

pub struct AfterRenderContext<'a, Scene> {
    pub gui:          &'a mut Gui,
    pub scene:        &'a mut Scene,
}

// Update results structs
// ----------------------

#[derive(Debug, Clone)]
pub enum UpdateResultAction {
    None, Redraw, Exit
}
impl UpdateResultAction {
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (UpdateResultAction::Exit, _) => UpdateResultAction::Exit,
            (_, UpdateResultAction::Exit) => UpdateResultAction::Exit,
            (UpdateResultAction::Redraw, _) => UpdateResultAction::Redraw,
            (_, UpdateResultAction::Redraw) => UpdateResultAction::Redraw,
            _ => UpdateResultAction::None,
        }
    }
}

pub struct InputUpdateResult {
    pub handled: bool,
    pub result: UpdateResultAction
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
            result: UpdateResultAction::None
        }
    }
}

// UpdaterModule
// -------------

pub trait UpdaterModule<Scene> {
    fn input(&mut self, context: &mut UpdateContext<Scene>) -> InputUpdateResult;
    fn update(&mut self, context: &mut UpdateContext<Scene>) -> UpdateResultAction;
    fn resize(&mut self, context: &mut ResizeContext<Scene>) -> UpdateResultAction;
    fn after_render(&mut self, state: &mut AfterRenderContext<Scene>);
}

// Updater
// -------

pub struct Updater<Scene> {
    modules: Vec<Box<dyn UpdaterModule<Scene>>>,
    pub update_cnt: u64,
    pub input_cnt: u64,
}

impl<Scene> Updater<Scene> {
    pub fn new() -> Self {
        Self {
            modules: vec![],
            update_cnt: 0,
            input_cnt: 0,
        }
    }
    
    pub fn with_module<M>(mut self, module: M) -> Self
    where
        M: UpdaterModule<Scene> + 'static
    {
        self.modules.push(Box::new(module));
        self
    }
    
    /// Invoked when input has changed
    #[profiler::function]
    pub fn input(&mut self, mut context: UpdateContext<Scene>) -> UpdateResultAction {
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
    pub fn update(&mut self, mut context: UpdateContext<Scene>) -> UpdateResultAction {
        let mut result = UpdateResultAction::None;
        for module in self.modules.iter_mut() {
            result = result.combine(module.update(&mut context));
        }
        self.update_cnt += 1;
        result
    }
    
    /// React to resize event
    #[profiler::function]
    pub fn resize(&mut self, mut context: ResizeContext<Scene>) -> UpdateResultAction {
        let mut result = UpdateResultAction::None;
        for module in self.modules.iter_mut() {
            result = result.combine(module.resize(&mut context));
        }
        result
    }
    
    #[profiler::function]
    pub fn after_render(&mut self, mut context: AfterRenderContext<Scene>) {
        for module in self.modules.iter_mut() {
            module.after_render(&mut context);
        }
    }
    
}
