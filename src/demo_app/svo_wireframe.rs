///! SVOWireframeRenderModule is module which renders a svo as wireframe

pub use wgpu::PushConstantRange;
pub use wgpu::util::DeviceExt;

use crate::{
    sdf::svo,
    demo_app::scene::Scene,
    framework::{
        gui::Gui,
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPass
        },
    },
};

use super::cube::CubeOutlinePipeline;

#[derive(Debug)]
pub struct SvoWireframeRenderModule {
    visible: bool,
    pipeline: CubeOutlinePipeline,
}

impl SvoWireframeRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        SvoWireframeRenderModule {
            visible: true,
            pipeline: CubeOutlinePipeline::new(context)
        }
    }
}

impl RenderModule<Scene> for SvoWireframeRenderModule {
    
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        // NOTE: For now this implementation just copies all SVO vertices from all geometries into a single buffer
        // -------------------------------------------------------------------------------------------------------
        
        self.visible = scene.display_toggles.show_wireframe;
        if !self.visible {
            return;
        }
        
        // Get all nodes from all valid node pools from all geometries with their node count
        let values: Vec<(u32, &svo::Svo)> = scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| {
                let Some(svo) = &geometry.svo else { return None; };
                let Some(cnt) = &svo.node_pool.count() else { return None; };
                Some((cnt.clone(), svo))
            })
            .collect();
        
        // Prepare command encoder
        let mut encoder = context.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SVO Render Module Encoder For copying vertices into node vertex buffer"),
        });
        
        // Lets ensure we have enough space in the buffer for all vertices by summing all node counts
        let total_count = values.iter().map(|(cnt, _)| cnt).sum::<u32>() as usize;
        self.pipeline.instance_buffer.resize(&context.gpu, total_count);
        
        // Copy all vertices into the buffer from all node pools
        let mut vertices_copied = 0;
        self.pipeline.instance_buffer.size = 0;
        { profiler::scope!("Pushing all vertices from SVO to svo wireframe renderer vertex buffer");
            encoder.push_debug_group("Copying vertices from node pool to svo renderer");
            values.iter().for_each(|(cnt, svo)| {
                profiler::scope!("SVO vertex buffer -> svo renderer vertex buffer");
                encoder.copy_buffer_to_buffer(
                    svo.node_pool.vertex_buffer(),
                    0,
                    &self.pipeline.instance_buffer.buffer,
                    vertices_copied as u64,
                    ((cnt.clone() as usize) * std::mem::size_of::<glam::Vec4>()) as u64
                );
                self.pipeline.instance_buffer.size += cnt.clone() as usize;
                vertices_copied += cnt.clone();
                self.pipeline.set_domain(svo.domain);
            });
            encoder.pop_debug_group();
        }
        
        // Submit command to queue
        profiler::call!(context.gpu.queue.submit(Some(encoder.finish())));
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        if !self.visible {
            return;
        }
        
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPass::Base { .. },
                render_pass
            } => {
                self.pipeline.render_on_pass(render_pass, &context.camera);
            },
            _ => {}
        }
    }
    
    fn finalize(&mut self) {}
}
