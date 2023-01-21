
use crate::{
    demo_app::scene::Scene,
    framework::{
        gui::Gui,
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPass,
        },
    }, warn,
};

use super::{SvoSDFBrickPipeline, SvoBrickSelectPipeline, BrickInstances};

///! This is main renderer of evaluated geometries

#[derive(Debug)]
pub struct SvoSdfBricksRenderModule {
    pipeline: SvoSDFBrickPipeline,
    brick_select_compute_pipeline: SvoBrickSelectPipeline,
    brick_instances: BrickInstances,
}

impl SvoSdfBricksRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            pipeline: SvoSDFBrickPipeline::new(context),
            brick_select_compute_pipeline: SvoBrickSelectPipeline::new(context),
            brick_instances: BrickInstances::new(&context.gpu, 1024),
        }
    }
}

impl RenderModule<Scene> for SvoSdfBricksRenderModule {
    
    /// Prepares list of nodes to be rendered in this frame.
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        
        // TODO: Render all SVOs
        let svo = scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| { geometry.svo.as_ref() })
            .take(1)
            .last();
        
        let Some(svo) = svo else {
            warn!("SvoSdfBricksRenderModule::prepare: No SVOs to render");
            return;
        };
        
        let Some(node_count) = svo.node_pool.count() else {
            warn!("SvoSdfBricksRenderModule::prepare: SVO node pool is empty on does nto have node count loaded from GPU");
            return;
        };
        
        self.brick_instances.clear_resize(&context.gpu, node_count as usize);
        self.brick_select_compute_pipeline.run(context, &svo, &self.brick_instances, scene.brick_level_break_size);
        
        {
            profiler::scope!("BrickInstances::load_count", pinned);
            // TODO: (SLOW) this will not be needed when we will use indirect draw
            // TODO: add node count to GUI display -> there has to be a global stat counter accessible even when scene is immutable
            self.brick_instances.load_count(&context.gpu)
        };
        
        self.pipeline.set_svo(&context.gpu, svo);
        self.pipeline.set_display_options(scene.display_toggles.brick_display_options);
        
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPass::Base { .. },
                render_pass,
            } => {
                self.pipeline.render_on_pass(render_pass, context, &self.brick_instances);
            }
            _ => {}
        }
    }

    fn finalize(&mut self) {}
}
