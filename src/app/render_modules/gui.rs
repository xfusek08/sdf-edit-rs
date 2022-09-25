/// This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use std::sync::Arc;

use egui::ClippedPrimitive;
use egui_wgpu::renderer::{RenderPass, ScreenDescriptor};

use crate::app::{
    gui::Gui,
    scene::Scene,
    rendering::{RenderModule, RenderContext},
};

pub struct GuiRenderer {
    egui_renderer: RenderPass,
    render_data: Option<RenderData>,
}

struct RenderData {
    paint_jobs: Arc<Vec<ClippedPrimitive>>,
    screen_descriptor: ScreenDescriptor,
}

// Construct this render module (a pipeline) from render context
impl GuiRenderer {
    
    #[profiler::function]
    pub fn new(context: &RenderContext) -> GuiRenderer {
        Self {
            egui_renderer: RenderPass::new(&context.gpu.device, context.surface_config.format, 1),
            render_data: None,
        }
    }
    
}

impl RenderModule for GuiRenderer {
    
    #[profiler::function]
    fn prepare(&mut self, gui: &Gui, _: &Scene, context: &RenderContext) {
        
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.surface_config.width, context.surface_config.height],
            pixels_per_point: context.scale_factor as f32,
        };
        
         if let Some(textures_delta) = gui.textures_delta.as_ref() {
            profiler::scope!("Update Textures");
            for (id, image_delta) in &textures_delta.set {
                profiler::scope!("Update Texture");
                self.egui_renderer.update_texture(
                    &context.gpu.device,
                    &context.gpu.queue,
                    *id,
                    image_delta,
                );
            }
        }
        
        profiler::call!(
            self.egui_renderer.update_buffers(
                &context.gpu.device,
                &context.gpu.queue,
                &**gui.paint_jobs,
                &screen_descriptor
            )
        );
        
        { profiler::scope!("store render data");
            self.render_data = Some(RenderData {
                paint_jobs: gui.paint_jobs.clone(),
                screen_descriptor,
            });
        }
        
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(&'a mut self, _: &'a RenderContext, render_pass: &mut wgpu::RenderPass<'pass>) {
        if let Some(data) = self.render_data.as_ref() {
            render_pass.push_debug_group("egu render pass");
            self.egui_renderer.execute_with_renderpass(
                render_pass,
                &data.paint_jobs,
                &data.screen_descriptor
            );
            render_pass.pop_debug_group();
        }
    }
    
    #[profiler::function]
    fn finalize(&mut self, gui: &mut Gui, _: &mut crate::app::scene::Scene) {
        if let Some(textures_delta) = gui.textures_delta.take() {
            for id in &textures_delta.free {
                self.egui_renderer.free_texture(id);
            }
        }
    }
    
}
