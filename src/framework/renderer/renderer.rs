use std::{sync::Arc, fmt::Debug};

use slotmap::{SlotMap, new_key_type};
use winit::window::Window;

use crate::{
    error,
    framework::{
        gpu,
        gui::Gui,
        camera::SceneWithCamera
    },
};

use super::{
    RenderPass,
    RenderModule,
    camera::Camera,
    RenderContext,
};

new_key_type! { pub struct RenderModuleID; }
new_key_type! { pub struct RenderPassID; }


#[derive(Debug)]
struct RegisteredRenderPass {
    pass:    RenderPass,
    modules: Vec<RenderModuleID>,
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
        
        let swapchain_capabilities = gpu.surface.get_capabilities(&gpu.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        
        // setup surface for rendering
        let surface_config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT, // texture will be used to draw on screen
            format:       swapchain_format,        // texture format - select first supported one
            present_mode: wgpu::PresentMode::Mailbox, // VSynch essentially - capping renders on display frame rate
            width:        window.inner_size().width,
            height:       window.inner_size().height,
            alpha_mode:   wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
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
    pub fn register_module<M, F>(&mut self, get_module: F) -> RenderModuleID
    where
        M: RenderModule<S> + 'static,
        F: FnOnce(&RenderContext) -> M,
    {
        self.modules.insert(Box::new(get_module(&self.context)))
    }
    
    pub fn register_render_pass<F>(&mut self, get_pass: F, modules: &[RenderModuleID]) -> RenderPassID
    where
        F: FnOnce(&RenderContext) -> RenderPass,
    {
        let pass = get_pass(&self.context);
        
        // Check if modules are registered, panic if not.
        for module_id in modules {
            if !self.modules.contains_key(*module_id) {
                panic!("Cannot register render pass: module {:?} is not registered", module_id);
            }
        }
        
        self.passes.insert(RegisteredRenderPass {
            pass,
            modules: modules.to_vec(),
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
            for RegisteredRenderPass { pass, .. } in self.passes.values_mut() {
                pass.resize(&self.context, scale_factor);
            }
        }
    }
    
    #[profiler::function(pinned)]
    pub fn prepare(&mut self, gui: &Gui, scene: &S) {
        
        // Update shared GPU resource outside of individual render module scopes
        
        // update camera
        // TODO: Only if camera is dirty?
        self.context.camera.update(&self.context.gpu.queue, scene.get_camera_rig().camera());
        
        // Prepare all modules
        // TODO: parallelize this
        self.modules
            .values_mut()
            .for_each(|m| m.prepare(gui, scene, &self.context));
    }
    
    #[profiler::function(pinned)]
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
            self.passes
                .values_mut()
                .for_each(|pass| {
                    profiler::scope!("One Render Pass execute");
                    let mut pass_context = pass.pass.start(&mut encoder, &view, &self.context);
                    pass.modules
                        .iter()
                        .filter_map(|m_id| {
                            let Some(module) = self.modules.get(*m_id) else {
                                error!("Render module not found: {:?} requested by pass:\n{:?}", m_id, pass);
                                return None;
                            };
                            Some(module)
                        })
                        .for_each(|m| {
                            profiler::scope!("One Module execute");
                            m.render(&self.context, &mut pass_context);
                        });
                });
        }
        
        profiler::call!(self.context.gpu.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
    }
    
    #[profiler::function]
    pub fn finalize(&mut self) {
        // TODO: Implement finalize
    }
    
}
