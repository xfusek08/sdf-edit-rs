///! SVOWireframeRenderModule is module which renders a svo as wireframe

pub use wgpu::PushConstantRange;
pub use wgpu::util::DeviceExt;

use crate::app::{
    renderer::{
        RenderContext,
        render_module::RenderModule,
        render_pass::{RenderPassAttachment, RenderPassContext}
    },
    state::State,
    sdf::svo::NodePool,
    pipelines::cube_outline::CubeOutlinePipeline,
};



#[derive(Debug)]
pub struct SVOWireframeRenderModule {
    pipeline: CubeOutlinePipeline,
}

impl SVOWireframeRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        SVOWireframeRenderModule {
            pipeline: CubeOutlinePipeline::new(context)
        }
    }
}

impl RenderModule for SVOWireframeRenderModule {
    
    #[profiler::function]
    fn prepare(&mut self, state: &State, context: &RenderContext) {
        // NOTE: For now this implementation just copies all SVO vertices from all geometries into a single buffer
        // -------------------------------------------------------------------------------------------------------
        
        // Get all nodes from all valid node pools from all geometries with their node count
        let values: Vec<(u32, &NodePool)> = state.scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| {
                if let Some(svo) = &geometry.svo {
                    if let Some(cnt) = &svo.node_pool.count() {
                        return Some((cnt.clone(), &svo.node_pool));
                    }
                }
                None
            })
            .collect();
        
        // Prepare command encoder
        let mut encoder = context.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SVO Render Module Encoder For copying vertices into node vertex buffer"),
        });
            
        // Lets ensure we have enough space in the buffer for all vertices by summing all node counts
        let total_count: u32 = values.iter().map(|(cnt, _)| cnt).sum();
        let total_count = total_count as usize;
        if total_count > self.pipeline.instance_buffer.capacity {
            profiler::scope!("Resizing node vertex buffer");
            encoder.push_debug_group("Resizing SVO Node Vertex Buffer");
            encoder.insert_debug_marker("Resizing node vertex buffer");
            self.pipeline.instance_buffer.resize(&context.gpu, total_count);
            encoder.pop_debug_group();
        }
        
        
        // Copy all vertices into the buffer from all node pools
        let mut vertices_copied = 0;
        self.pipeline.instance_buffer.size = 0;
        { profiler::scope!("Pushing all vertices from SVO to svo wireframe renderer vertex buffer");
            encoder.push_debug_group("Copying vertices from node pool to svo renderer");
            values.iter().for_each(|(cnt, node_pool)| {
                { profiler::scope!("SVO vertex buffer -> svo renderer vertex buffer");
                    encoder.copy_buffer_to_buffer(
                        node_pool.vertex_buffer(),
                        0,
                        &self.pipeline.instance_buffer.buffer,
                        vertices_copied as u64,
                        (cnt.clone() as usize * std::mem::size_of::<glam::Vec4>()) as u64
                    );
                }
                self.pipeline.instance_buffer.size += cnt.clone() as usize;
                vertices_copied += cnt.clone();
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
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPassAttachment::Base { .. },
                render_pass
            } => {
                self.pipeline.render_on_pass(render_pass, &context.camera);
            },
            _ => {}
        }
    }
    
    fn finalize(&mut self) {}
}
