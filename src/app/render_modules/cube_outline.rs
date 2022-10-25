use crate::app::{
    objects::cube::CubeOutlineComponent,
    pipelines::cube_outline::CubeOutlinePipeline,
    renderer::{
        render_module::RenderModule,
        render_pass::{RenderPassAttachment, RenderPassContext},
        RenderContext,
    },
};

///! This module renders all cube outlines in scene.

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

impl RenderModule for CubeOutlineRenderModule {
    #[profiler::function]
    fn prepare(&mut self, state: &crate::app::state::State, context: &RenderContext) {
        let instances: Vec<CubeOutlineComponent> = {
            profiler::scope!("Collect cube outline instances from world");
            state
                .scene
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
        render_pass_context: &mut crate::app::renderer::render_pass::RenderPassContext<'pass>,
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
