use wgpu::RenderPass;
use crate::app::{scene::Scene, gui::Gui};
use super::RenderContext;


pub trait RenderModule {
    fn prepare(&mut self, gui: &Gui, scene: &Scene, context: &RenderContext);
    
    /// Render this (prepared) module
    ///  - `'a: 'pass` (`'a` outlives `'pass`) meaning that this render module lives longer than the render pass
    fn render<'pass, 'a: 'pass>(&'a mut self, context: &'a RenderContext, render_pass: &mut RenderPass<'pass>);
    
    // Finalization step (after rendering) which can alter scene state meant to unflag dirty components as clean (prepared)
    fn finalize(&mut self, gui: &mut Gui, scene: &mut Scene);
}
