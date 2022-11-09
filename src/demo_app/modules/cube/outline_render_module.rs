
use crate::{
    demo_app::scene::Scene,
    framework::{
        gui::Gui,
        renderer::{
            RenderContext,
            RenderModule,
            RenderPassContext,
            RenderPassAttachment,
        },
    },
};

use super::{
    CubeOutlinePipeline,
    CubeOutlineComponent,
};

#[derive(Debug)]
pub struct CubeOutlineRenderModule {
    pipeline: CubeOutlinePipeline,
}

impl CubeOutlineRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        CubeOutlineRenderModule {
            pipeline: CubeOutlinePipeline::new(context),
        }
    }
}

impl RenderModule<Scene> for CubeOutlineRenderModule {
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        let instances: Vec<CubeOutlineComponent> = {
            profiler::scope!("Collect cube outline instances from world");
            scene
                .world
                .query::<&CubeOutlineComponent>()
                .iter()
                .map(|(_, cube)| cube.clone())
                .collect()
        };
        
        profiler::call!(self
            .pipeline
            .instance_buffer
            .queue_update(&context.gpu, &instances));
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPassAttachment::Base { .. },
                render_pass,
            } => {
                self.pipeline.render_on_pass(render_pass, &context.camera);
            }
            _ => {}
        }
    }
    
    fn finalize(&mut self) {}
}
