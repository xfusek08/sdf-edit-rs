use std::{sync::Arc, ops::Deref};

use crate::{app::{
    renderer::{
        render_pass::{RenderPassContext, RenderPassAttachment},
        render_module::RenderModule,
        RenderContext,
    },
    pipelines::svo_solid_brick::SvoSolidBrickPipeline,
}, sdf::svo::{Svo, self}};

///! This is main renderer of evaluated geometries

#[derive(Debug)]
pub struct SvoSolidBricksRenderModule {
    pipeline: SvoSolidBrickPipeline,
    svo: Option<Arc<Svo>>,
}

impl SvoSolidBricksRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            pipeline: SvoSolidBrickPipeline::new(context),
            svo: None,
        }
    }
}

impl RenderModule for SvoSolidBricksRenderModule {
    #[profiler::function]
    fn prepare(&mut self, state: &crate::app::state::State, context: &RenderContext) {
        // TODO: Select which nodes will be renderer for given svo
        
        // NOTE: For now only first geometry is rendered
        let svo = state.scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| { geometry.svo.as_ref().cloned() })
            .take(1)
            .last();
            
        if let Some(svo) = svo {
            let level = svo.levels.get(state.scene.tmp_evaluator_config.render_level as usize);
            
            if let Some(svo::Level { node_count, start_index }) = level {
                let end = start_index + node_count;
                let new_data: Vec<u32> = (*start_index..end).collect();
                self.pipeline.brick_instance_buffer.queue_update(
                    &context.gpu,
                    &new_data,
                )
            }
            
            self.pipeline.set_svo(&context.gpu, svo.deref());
            self.svo = Some(svo);
        }
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
                self.pipeline.render_on_pass(render_pass, context);
            }
            _ => {}
        }
    }

    fn finalize(&mut self) {}
}
