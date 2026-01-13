///! This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs
use egui::ClippedPrimitive;
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};

use crate::framework::renderer::{RenderContext, RenderModule, RenderPass, RenderPassContext};

use super::{Gui, GuiDataToRender};

struct RenderData {
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
}

pub struct GuiRenderModule {
    egui_renderer: Renderer,
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
            egui_renderer: Renderer::new(
                &context.gpu.device,
                context.surface_config.format,
                None,
                1,
            ),
            render_data: None,
        }
    }
}

impl<Scene> RenderModule<Scene> for GuiRenderModule {
    #[profiler::function]
    fn prepare(&mut self, gui: &Gui, _: &Scene, context: &RenderContext) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.surface_config.width, context.surface_config.height],
            pixels_per_point: context.scale_factor as f32,
        };

        if let Some(GuiDataToRender {
            textures_delta,
            shapes,
        }) = gui.data_to_render.as_ref()
        {
            // Gui Changed - process update

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

            let clipped_primitives = {
                profiler::scope!("Recalculate shapes to paint jobs");
                gui.egui_ctx.tessellate(shapes.clone())
            };

            {
                profiler::scope!("Update egui buffers using encoder");

                let mut encoder = {
                    profiler::scope!("Create encoder");
                    context
                        .gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Egui command encoder"),
                        })
                };

                let buffers = {
                    profiler::scope!("Update egui buffers");
                    self.egui_renderer.update_buffers(
                        &context.gpu.device,
                        &context.gpu.queue,
                        &mut encoder,
                        &clipped_primitives,
                        &screen_descriptor,
                    )
                };

                let encoded = {
                    profiler::scope!("Finish encoder");
                    encoder.finish()
                };

                {
                    profiler::scope!("Submit encoder");
                    context
                        .gpu
                        .queue
                        .submit(buffers.into_iter().chain(std::iter::once(encoded)));
                }
            }

            {
                profiler::scope!("Free textures which are no longer used");
                for id in &textures_delta.free {
                    self.egui_renderer.free_texture(id);
                }
            }

            self.render_data = Some(RenderData {
                clipped_primitives,
                screen_descriptor,
            });
        } else if let Some(render_data) = self.render_data.as_mut() {
            // Gui didn't change - but state exists - update screen descriptor
            render_data.screen_descriptor = screen_descriptor;
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
                attachment: RenderPass::Gui { .. },
                render_pass,
            } => {
                if let Some(data) = self.render_data.as_ref() {
                    render_pass.push_debug_group("egu render pass");
                    self.egui_renderer.render(
                        render_pass,
                        &data.clipped_primitives,
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
