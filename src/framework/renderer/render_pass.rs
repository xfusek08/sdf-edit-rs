
use crate::framework::gpu;

use super::RenderContext;

#[derive(Debug)]
pub enum RenderPass {
    /// Main render pass drawing color values to the screen, using depth buffer
    Base {
        clear_color: wgpu::Color,
        depth_texture: gpu::DepthStencilTexture,
    },
    
    /// Render pass drawing gui elements to the screen
    Gui {
        
    },
}

#[derive(Debug)]
pub struct RenderPassContext<'pass> {
    pub attachment:  &'pass RenderPass,
    pub render_pass: wgpu::RenderPass<'pass>,
}

// Construction
impl RenderPass {
    
    pub fn base(context: &RenderContext) -> Self {
        Self::Base {
            #[cfg(feature = "white_bg")]
            clear_color: wgpu::Color::WHITE,
            #[cfg(not(feature = "white_bg"))]
            clear_color:  wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            depth_texture: gpu::DepthStencilTexture::new(
                "Base Pass Depth texture",
                &context.gpu.device,
                &context.surface_config
            ),
        }
    }
    
    pub fn gui(context: &RenderContext) -> Self {
        Self::Gui {
            
        }
    }
    
}

impl RenderPass {
    pub fn resize(&mut self, context: &RenderContext, scale_factor: f64) {
        match self {
            Self::Base { depth_texture, .. } => {
                *depth_texture = gpu::DepthStencilTexture::new(
                    "Base Pass Depth texture",
                    &context.gpu.device,
                    &context.surface_config
                );
            },
            Self::Gui { .. } => {
                
            },
        }
    }
    
    pub fn start<'pass>(
        &'pass self,
        encoder: &'pass mut wgpu::CommandEncoder,
        view: &'pass wgpu::TextureView,
        context: &'pass RenderContext,
    ) ->  RenderPassContext<'pass> {
        match self {
            Self::Base {
                clear_color,
                depth_texture
            } => RenderPassContext {
                attachment: self,
                render_pass: encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    
                    // Color frame buffer
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color.clone()),
                            store: true
                        }
                    })],
                    
                    // Depth buffer to use in depth testing in this pass (if any in context)depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.texture().view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                })
            },
            
            Self::Gui {  } =>  RenderPassContext {
                attachment: self,
                render_pass: encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Gui Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true
                        }
                    })],
                    depth_stencil_attachment:None,
                }),
            },
        }
    }
}
