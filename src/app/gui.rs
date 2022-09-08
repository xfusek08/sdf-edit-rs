use egui::Window;
use winit_input_helper::WinitInputHelper;

use super::clock::Tick;
use super::rendering::RenderContext;
use super::application::UpdateResult;

pub struct Gui;

impl Gui {
    #[profiler::function]
    pub fn new(window: &Window, render_context: &RenderContext) -> Self {
        Self {}
    }
    
    #[profiler::function]
    pub fn render(&mut self, render_context: &RenderContext) {
        
    }
    
    #[profiler::function]
    pub fn input(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        UpdateResult::Wait
    }

    #[profiler::function]
    pub fn update(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        UpdateResult::Wait
    }
}
