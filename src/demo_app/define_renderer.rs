
use crate::framework::{
    application::Context,
    gui::GuiRenderModule,
    renderer::{Renderer, RenderPass},
};

use super::{
    scene::Scene,
    modules::{
        line::LineRenderModule,
        cube::CubeOutlineRenderModule,
        svo_sdf_brick::SvoSdfBricksRenderModule,
        SvoWireframeRenderModule,
    },
};


pub fn define_renderer(context: &Context) -> Renderer<Scene> {
    let mut renderer = Renderer::new(context.gpu.clone(), context.window);
    
    // load modules
    let line_module = renderer.register_module(LineRenderModule::new);
    let cube_outline = renderer.register_module(CubeOutlineRenderModule::new);
    let svo_wireframe_module = renderer.register_module(SvoWireframeRenderModule::new);
    let svo_sdf_brick_module = renderer.register_module(SvoSdfBricksRenderModule::new);
    let gui_module = renderer.register_module(GuiRenderModule::new);
    
    // passes are executed in order of their registration
    renderer.register_render_pass(RenderPass::base, &[
        line_module,
        cube_outline,
        svo_sdf_brick_module,
        svo_wireframe_module,
    ]);
    
    renderer.register_render_pass(RenderPass::gui, &[
        gui_module
    ]);
    
    renderer
}
