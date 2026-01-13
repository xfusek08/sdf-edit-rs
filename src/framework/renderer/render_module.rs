use std::fmt::Debug;

use crate::framework::gui::Gui;

use super::{RenderContext, RenderPassContext};

pub trait RenderModule<Scene>: Debug {
    fn prepare(&mut self, gui: &Gui, scene: &Scene, context: &RenderContext);

    /// Render this (prepared) module
    ///  - `'a: 'pass` (`'a` outlives `'pass`) meaning that this render module lives longer than the render pass
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    );

    // Finalization step after rendering to give the module a chance to clean up
    fn finalize(&mut self);
}
