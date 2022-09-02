use winit::window::Window;
use crate::app::scene::Scene;
use super::{
    RenderContext,
    RenderModule,
};

pub struct Renderer {
    context: RenderContext,
    modules: Vec<Box<dyn RenderModule>>, // this means that renderer is owner of all instances in this vector and those cannot outlive the renderer.
}

// Builder
impl Renderer {
    
    /// Creates a new renderer instance for window (initialize rendering context)
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        Renderer {
            context: RenderContext::new(window).await,
            modules: vec![],
        }
    }
    
    /// Adds a new render module to the renderer
    pub fn with_module<R>(mut self) -> Self
        where
            R: RenderModule + for<'a> From<&'a RenderContext> + 'static
    {
        let module: R = R::from(&self.context);
        self.modules.push(Box::new(module));
        self
    }
    
    /// Resize Rendering context
    #[profiler::function]
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.context.surface_config.width = size.width;
            self.context.surface_config.height = size.height;
            self.context.surface.configure(&self.context.device, &self.context.surface_config);
        }
    }
    
    /// Search scene for changes and update corresponding data on GPU for rendering
    #[profiler::function]
    pub fn prepare(&mut self, scene: &Scene) {
        // update camera
        //  - TODO: if camera is dirty?
        self.context.camera.update(&self.context.queue, &scene.camera);
        
        // Update each module
        //  - TODO: could be parallelized
        for module in &mut self.modules {
            module.prepare(&self.context, scene);
        }
    }
    
    /// Draw  on screen
    #[profiler::function]
    pub fn render(&mut self)  {
        
        // ask surface to provide us a texture we will draw into
        let output = profiler::call!(
            self.context
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
            self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        );
        
        // prepare a render pass command in the encoder
        { profiler::scope!("Prepare render pass");
            // ⬇ Make render_pass mutable to be able add a pipeline to it
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // ⬇ set target to which gpu will be drawing into - might be one or more textures
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
                // ⬇ We do not use that for now
                depth_stencil_attachment: None
            });
            
            for module in &mut self.modules {
                module.render(&self.context, &mut render_pass);
            }
        } // drop render_pass here - because commands must not be borrowed before calling `finish()` on encoder
        
        profiler::call!(self.context.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
    }
}
