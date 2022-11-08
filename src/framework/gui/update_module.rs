
use crate::framework::updater::{
    UpdaterModule,
    UpdateContext,
    InputUpdateResult,
    UpdateResultAction,
    ResizeContext,
    AfterRenderContext
};

use super::GuiDataToRender;

pub struct GuiUpdateModule<F, Scene>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    draw_gui: F,
    _phantom: std::marker::PhantomData<Scene>,
}

impl<F, Scene> GuiUpdateModule<F, Scene>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    pub fn new(draw_gui: F) -> Self {
        Self {
            draw_gui,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, Scene> UpdaterModule<Scene> for GuiUpdateModule<F, Scene>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    
    fn input(&mut self, context: &mut UpdateContext<Scene>) -> InputUpdateResult {
        InputUpdateResult::default()
    }
    
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<Scene>) -> UpdateResultAction {
        let gui = &mut context.gui;
        let scene = &mut context.scene;
        
        let raw_input = profiler::call!(
            gui.egui_winit.take_egui_input(context.window)
        );
        
        // Run gui
        let egui::FullOutput {
            platform_output,
            repaint_after: _,
            textures_delta,
            shapes,
        } = profiler::call!(
            gui.egui_ctx.run(raw_input, |egui_ctx| (self.draw_gui)(egui_ctx, scene))
        );
        
        // Update window state (mainly to change cursors)
        profiler::call!(
            gui.egui_winit.handle_platform_output(context.window, &gui.egui_ctx, platform_output)
        );
        
        // Check if event should propagate to the scene
        let _ = gui.egui_ctx.is_using_pointer() || gui.egui_ctx.wants_keyboard_input();
        
        // Update textures, if any. Renderer is responsible for taking (removing gui.textures_delta) when it renders
        let textures_delta = if let Some(mut data) = gui.data_to_render.take() {
            data.textures_delta.append(textures_delta);
            data.textures_delta
        } else { textures_delta  };
        
        gui.data_to_render = Some(GuiDataToRender { textures_delta, shapes });
        
        UpdateResultAction::None
    }
    
    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext<Scene>) -> UpdateResultAction {
        context.gui.egui_ctx.set_pixels_per_point(context.scale_factor as f32);
        UpdateResultAction::None
    }
    
    /// After frame is renderer clean render data
    #[profiler::function]
    fn after_render(&mut self, state: &mut AfterRenderContext<Scene>) {
        state.gui.data_to_render = None;
    }
}
