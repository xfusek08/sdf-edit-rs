use wgpu::RenderPass;
use winit::window::Window;
use crate::app::{scene::Scene, gui::Gui};

use super::gpu::{GPUCamera, GPUContext, texture::DepthStencilTexture};

/// A GPU context for rendering purposes
#[derive(Debug)]
pub struct RenderContext<'a> {
    
    /// A GPU context which is shared with whole application
    pub gpu: &'a GPUContext,
    
    /// Configuration of surface is renderers responsibility
    pub surface_config: wgpu::SurfaceConfiguration,
    
    /// A part of surface configuration
    pub scale_factor: f64,
    
    /// Shared GOU resources provided for all render modules
    
    /// A camera GPU resource
    pub camera: GPUCamera,
    
    pub depth_texture: Option<DepthStencilTexture>,
    
}

pub trait RenderModule {
    fn prepare(&mut self, gui: &Gui, scene: &Scene, context: &RenderContext);
    
    /// Render this (prepared) module
    ///  - `'a: 'pass` (`'a` outlives `'pass`) meaning that this render module lives longer than the render pass
    fn render<'pass, 'a: 'pass>(&'a mut self, context: &'a RenderContext, render_pass: &mut RenderPass<'pass>);
    
    // Finalization step (after rendering) which can alter scene state meant to unflag dirty components as clean (prepared)
    fn finalize(&mut self, gui: &mut Gui, scene: &mut Scene);
}

pub struct Renderer<'a> {
    context:        RenderContext<'a>,
    modules:        Vec<Box<dyn RenderModule>>, // this means that renderer is owner of all instances in this vector and those cannot outlive the renderer.
    pub render_cnt: u64,
}

impl<'a> Renderer<'a> {
    
    /// Creates a new renderer instance for window (initialize rendering context)
    #[profiler::function]
    pub fn new(gpu: &'a GPUContext, window: &Window) -> Renderer<'a> {
        
        // setup surface for rendering
        let surface_config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,             // texture will be used to draw on screen
            format:       gpu.surface.get_supported_formats(&gpu.adapter)[0], // texture format - select first supported one
            present_mode: wgpu::PresentMode::Fifo,                            // VSynch essentially - capping renders on display frame rate
            width:        window.inner_size().width,
            height:       window.inner_size().height,
        };
        gpu.surface.configure(&gpu.device, &surface_config);
        
        // let depth_texture = DepthStencilTexture::new("Depth stencil texture", &gpu.device, &surface_config);
        
        Renderer {
            context: RenderContext {
                gpu,
                surface_config,
                scale_factor:  window.scale_factor(),
                camera:        GPUCamera::new(0, &gpu.device),
                // depth_texture: Some(depth_texture),
                depth_texture: None,
            },
            modules:    vec![],
            render_cnt: 0,
        }
    }
    
    /// Adds a new render module to the renderer
    pub fn with_module<M, F>(mut self, get_module: F) -> Self
        where
            M: RenderModule + 'static,
            F: FnOnce(&RenderContext) -> M,
    {
        let module = get_module(&self.context);
        self.modules.push(Box::new(module));
        self
    }
    
    /// Resize Rendering context
    #[profiler::function]
    pub fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>, scale_factor: f64) {
        if size.width > 0 && size.height > 0 {
            // update surface config
            self.context.surface_config.width = size.width;
            self.context.surface_config.height = size.height;
            
            // update scale factor
            self.context.scale_factor = scale_factor;

            // re-configure surface with updated config
            self.context.gpu.surface.configure(&self.context.gpu.device, &self.context.surface_config);
            
            // update (create new actually) depth texture for screen new resolution
            if self.context.depth_texture.is_some() {
                self.context.depth_texture = Some(DepthStencilTexture::new("Depth stencil texture", &self.context.gpu.device, &self.context.surface_config));
            }
        }
    }
    
    /// Search scene for changes and update corresponding data on GPU for rendering
    #[profiler::function]
    pub fn prepare(&mut self, gui: &Gui, scene: &Scene) {
        // update camera
        //  - TODO: Only if camera is dirty?
        self.context.camera.update(&self.context.gpu.queue, &scene.camera);
        
        // Update each module
        //  - TODO: could be parallelized
        for module in &mut self.modules {
            module.prepare(gui, scene, &self.context);
        }
    }
    
    /// Draw  on screen
    #[profiler::function]
    pub fn render(&mut self)  {
        
        // ask surface to provide us a texture we will draw into
        let output = profiler::call!(
            self.context.gpu
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        );
        
        // View on surface texture understandable by RenderPassColorAttachment
        let view = profiler::call!(
            output.texture.create_view(&wgpu::TextureViewDescriptor::default())
        );
        
        // Create an encoder for building a GPU commands for this frame
        let mut encoder = profiler::call!(
            self.context.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        );
        
        // Prepare a render pass command in the encoder
        // Make render_pass mutable to be able add a pipeline to it
        {
            let mut render_pass = {
                profiler::scope!("Create render pass");
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    
                    // Frame buffer to draw colors into in this pass
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, // <- Draw into our surface texture view
                        resolve_target: None, // <- Final resolved output (None -> view)
                        ops: wgpu::Operations {
                            // ⬇ What to do with previous frame colors? -> Clear with a solid opaque color
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            // ⬇ store rendered data to view?
                            store: true
                        }
                    })],
                    
                    // Depth buffer to use in depth testing in this pass (if any in context)
                    depth_stencil_attachment: match self.context.depth_texture.as_ref() {
                        Some(depth_texture) => Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture.texture().view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                        None => None,
                    },
                    
                })
            };
            
            {
                profiler::scope!("Render modules onto render pass");
                for module in &mut self.modules {
                    module.render(&self.context, &mut render_pass);
                }
            }
        } // drop render_pass here - because commands must not be borrowed before calling `finish()` on encoder
        
        profiler::call!(self.context.gpu.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
        
        self.render_cnt += 1;
    }
    
    /// Finalize rendering and update scene state - unflag dirty components as clean (prepared)
    #[profiler::function]
    pub fn finalize(&mut self, gui: &mut Gui, scene: &mut Scene) {
        for module in &mut self.modules {
            module.finalize(gui, scene);
        }
    }
}
