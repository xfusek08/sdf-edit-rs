use std::{sync::Arc, ops::Deref};

use crate::{
    demo_app::scene::Scene,
    sdf::svo::{
        self,
        Svo
    },
    framework::{
        gui::Gui,
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPassAttachment
        },
    },
};

use super::SvoSDFBrickPipeline;

///! This is main renderer of evaluated geometries

#[derive(Debug)]
pub struct SvoSdfBricksRenderModule {
    pipeline: SvoSDFBrickPipeline,
    svo: Option<Arc<Svo>>,
}

impl SvoSdfBricksRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            pipeline: SvoSDFBrickPipeline::new(context),
            svo: None,
        }
    }
}

impl RenderModule<Scene> for SvoSdfBricksRenderModule {
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        // TODO: Select which nodes will be renderer for given svo
        
        // NOTE: For now only first geometry is rendered
        let svo = scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| { geometry.svo.as_ref().cloned() })
            .take(1)
            .last();
            
        if let Some(svo) = svo {
            let level = svo.levels.get(scene.tmp_evaluator_config.render_level as usize);
            
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
        render_pass_context: &mut RenderPassContext<'pass>,
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
