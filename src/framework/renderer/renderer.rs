use std::{sync::Arc, fmt::Debug};

use slotmap::{SlotMap, new_key_type};
use winit::window::Window;

use crate::framework::{gpu, gui::Gui, camera::SceneWithCamera};

use super::{
    RenderPassAttachment,
    RenderModule,
    camera::Camera,
    RenderContext,
};

new_key_type! { pub struct RenderModuleID; }
new_key_type! { pub struct RenderPassID; }


#[derive(Debug)]
struct RegisteredRenderPass {
    attachment: RenderPassAttachment,
    modules:    Vec<RenderModuleID>,
}

#[derive(Debug)]
pub struct Renderer<S: SceneWithCamera> {
    context: RenderContext,
    modules: SlotMap<RenderModuleID, Box<dyn RenderModule<S>>>,
    passes:  SlotMap<RenderPassID, RegisteredRenderPass>,
}

// Renderer construction methods
impl<S: SceneWithCamera> Renderer<S> {
    pub fn new(gpu: Arc<gpu::Context>, window: &Window) -> Self {
        // setup surface for rendering
        let surface_config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,             // texture will be used to draw on screen
            format:       gpu.surface.get_supported_formats(&gpu.adapter)[0], // texture format - select first supported one
            present_mode: wgpu::PresentMode::Fifo,                            // VSynch essentially - capping renders on display frame rate
            width:        window.inner_size().width,
            height:       window.inner_size().height,
            // alpha_mode:   CompositeAlphaMode::PostMultiplied, // TODO: wgpu 0.14
        };
        gpu.surface.configure(&gpu.device, &surface_config);
        
        Self {
            context: RenderContext {
                gpu: gpu.clone(),
                surface_config,
                scale_factor:  window.scale_factor(),
                camera:        Camera::new(0, &gpu.device),
            },
            modules: SlotMap::with_key(),
            passes:  SlotMap::with_key(),
        }
    }
    
    /// Adds a new render module to the renderer
    pub fn add_module<M, F>(&mut self, get_module: F) -> RenderModuleID
        where
            M: RenderModule<S> + 'static,
            F: FnOnce(&RenderContext) -> M,
    {
        let module = get_module(&self.context);
        self.modules.insert(Box::new(module))
    }
    
    pub fn set_render_pass<F>(&mut self, get_pass: F, modules: &[RenderModuleID]) -> RenderPassID
        where
            F: FnOnce(&RenderContext) -> RenderPassAttachment,
    {
        let pass = get_pass(&self.context);
        
        // Check if modules are registered, panic if not.
        for module in modules {
            if !self.modules.contains_key(*module) {
                panic!("\
                    Cannot set render pass:\n\
                        {:?}\n\
                        Render module is not registered: \n\
                        {:?}\n\
                ", pass, module);
            }
        }
        
        self.passes.insert(RegisteredRenderPass {
            attachment: pass,
            modules:    modules.to_vec(),
        })
    }
}

// renderer runtime methods
impl<S: SceneWithCamera> Renderer<S> {
    
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
            
            // resize all passes
            for pass in self.passes.values_mut() {
                pass.attachment.resize(&self.context, scale_factor);
            }
        }
    }
    
    #[profiler::function]
    pub fn prepare(&mut self, gui: &Gui, scene: &S) {
        
        // Update shared GPU resource outside of individual render module scopes
        
        // update camera
        // TODO: Only if camera is dirty?
        self.context.camera.update(&self.context.gpu.queue, &scene.get_camera());
        
        // Prepare all modules
        // TODO: parallelize this
        for module in self.modules.values_mut() {
            module.prepare(gui, scene, &self.context);
        }
    }
    
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
        
        // for each render pass, call render
        
        { profiler::scope!("Render Passes");
            for pass in self.passes.values_mut() {
                profiler::scope!("One Render Pass execute");
                let mut render_pass_context = pass.attachment.start(&mut encoder, &view, &self.context);
                for module_id in pass.modules.iter() {
                    profiler::scope!("One Module execute");
                    let module = self.modules.get(*module_id).unwrap();
                    module.render(&self.context, &mut render_pass_context);
                }
            }
        }
        
        profiler::call!(self.context.gpu.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
    }
    
    #[profiler::function]
    pub fn finalize(&mut self) {
        // TODO: Implement finalize
    }
    
}
