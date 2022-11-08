///! This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use egui::ClippedPrimitive;
use egui_wgpu::renderer::{RenderPass, ScreenDescriptor};

use crate::framework::renderer::{
    RenderContext,
    RenderModule,
    RenderPassContext,
    RenderPassAttachment
};

use super::{
    GuiDataToRender,
    Gui
};

struct RenderData {
    paint_jobs: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
}

pub struct GuiRenderModule {
    egui_renderer: RenderPass,
    render_data: Option<RenderData>,
}
impl std::fmt::Debug for GuiRenderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GUIRenderModule").finish()
    }
}

// Construct this render module (a pipeline) from render context
impl GuiRenderModule {
    #[profiler::function]
    pub fn new(context: &RenderContext) -> GuiRenderModule {
        Self {
            egui_renderer: RenderPass::new(&context.gpu.device, context.surface_config.format, 1),
            render_data: None,
        }
    }
}

impl<Scene> RenderModule<Scene> for GuiRenderModule {
    #[profiler::function]
    fn prepare(&mut self, gui: &Gui, scene: &Scene, context: &RenderContext) {
        if let Some(GuiDataToRender {
            textures_delta,
            shapes,
        }) = gui.data_to_render.as_ref() {
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [context.surface_config.width, context.surface_config.height],
                pixels_per_point: context.scale_factor as f32,
            };

            {
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

            let paint_jobs = {
                profiler::scope!("Recalculate shapes to paint jobs");
                gui.egui_ctx.tessellate(shapes.clone())
            };

            {
                profiler::scope!("Update egui buffers");
                self.egui_renderer.update_buffers(
                    &context.gpu.device,
                    &context.gpu.queue,
                    &paint_jobs,
                    &screen_descriptor,
                )
            }

            {
                profiler::scope!("Free textures which are no longer used");
                for id in &textures_delta.free {
                    self.egui_renderer.free_texture(id);
                }
            }

            self.render_data = Some(RenderData {
                paint_jobs,
                screen_descriptor,
            });
        }
    }

    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        _: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPassAttachment::Gui { .. },
                render_pass,
            } => {
                if let Some(data) = self.render_data.as_ref() {
                    render_pass.push_debug_group("egu render pass");
                    self.egui_renderer.execute_with_renderpass(
                        render_pass,
                        &data.paint_jobs,
                        &data.screen_descriptor,
                    );
                    render_pass.pop_debug_group();
                }
            }
            _ => {}
        }
    }

    fn finalize(&mut self) {}
}
