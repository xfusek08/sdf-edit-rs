use crate::framework::updater::{
    AfterRenderContext, InputUpdateResult, ResizeContext, UpdateContext, UpdateResultAction,
    UpdaterModule,
};

use super::GuiDataToRender;

/// Module that runs GUI

pub trait GuiModule<Scene> {
    fn gui_window(&mut self, scene: &mut Scene, egui_ctx: &egui::Context);
    fn gui_section(&mut self, scene: &mut Scene, ui: &mut egui::Ui);
}

/// Updater module for GUI running gui modules passed to constructor.
pub struct GuiUpdateModule<Scene> {
    modules: Vec<Box<dyn GuiModule<Scene>>>,
}

impl<Scene> GuiUpdateModule<Scene> {
    pub fn new(modules: Vec<Box<dyn GuiModule<Scene>>>) -> Self {
        Self { modules }
    }
}

impl<Scene> UpdaterModule<Scene> for GuiUpdateModule<Scene> {
    fn input(&mut self, context: &mut UpdateContext<Scene>) -> InputUpdateResult {
        InputUpdateResult::default()
    }

    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<Scene>) -> UpdateResultAction {
        let gui = &mut context.gui;
        let scene = &mut context.scene;

        let raw_input = profiler::call!(gui.egui_winit.take_egui_input(context.window));

        // Run gui
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = profiler::call!(gui.egui_ctx.run(raw_input, |egui_ctx| {
            egui::Window::new("Gui modules")
                .default_pos(egui::Pos2::new(0.0, 0.0))
                .show(egui_ctx, |ui| {
                    for module in self.modules.iter_mut() {
                        ui.separator();
                        module.gui_section(scene, ui);
                    }
                });
            for module in self.modules.iter_mut() {
                module.gui_window(scene, egui_ctx);
            }
        }));

        // Update window state (mainly to change cursors)
        profiler::call!(gui.egui_winit.handle_platform_output(
            context.window,
            &gui.egui_ctx,
            platform_output
        ));

        // Check if event should propagate to the scene
        let _ = gui.egui_ctx.is_using_pointer() || gui.egui_ctx.wants_keyboard_input();

        // Update textures, if any. Renderer is responsible for taking (removing gui.textures_delta) when it renders
        let textures_delta = if let Some(mut data) = gui.data_to_render.take() {
            data.textures_delta.append(textures_delta);
            data.textures_delta
        } else {
            textures_delta
        };

        gui.data_to_render = Some(GuiDataToRender {
            textures_delta,
            shapes,
        });

        if repaint_after.is_zero() {
            UpdateResultAction::Redraw
        } else {
            UpdateResultAction::None
        }
    }

    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext<Scene>) -> UpdateResultAction {
        context
            .gui
            .egui_ctx
            .set_pixels_per_point(context.scale_factor as f32);
        UpdateResultAction::Redraw
    }

    /// After frame is renderer clean render data
    #[profiler::function]
    fn after_render(&mut self, state: &mut AfterRenderContext<Scene>) {
        state.gui.data_to_render = None;
    }
}
