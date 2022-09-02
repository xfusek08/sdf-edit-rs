use wgpu::RenderPass;
use crate::app::scene::Scene;
use super::RenderContext;


pub trait RenderModule {
    fn prepare(&mut self, context: &RenderContext, scene: &Scene);
    
    /// Render this (prepared) module
    ///  - `'a: 'pass` (`'a` outlives `'pass`) meaning that this render module lives longer than the render pass
    fn render<'pass, 'a: 'pass>(&'a mut self, context: &'a RenderContext, render_pass: &mut RenderPass<'pass>);
}
