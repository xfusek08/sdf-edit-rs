
use crate::app::{
    gui::GuiDataToRender,
    application::ControlFlowResultAction,
    updating::{UpdateContext, UpdaterModule, InputUpdateResult, ResizeContext}, state::{State, Scene},
};

pub struct GuiUpdater<F>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    draw_gui: F,
}

impl<F> GuiUpdater<F>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    pub fn new(draw_gui: F) -> Self {
        Self {
            draw_gui
        }
    }
}

impl<F> UpdaterModule for GuiUpdater<F>
where
    F: Fn(&egui::Context, &mut Scene) -> (),
{
    
    fn input(&mut self, _: &mut UpdateContext) -> InputUpdateResult {
        InputUpdateResult::default()
    }
    
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        let State { gui, scene, ..} = context.state;
        
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
        
        ControlFlowResultAction::None
    }
    
    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction {
        let gui = &mut context.state.gui;
        gui.egui_ctx.set_pixels_per_point(context.scale_factor as f32);
        ControlFlowResultAction::None
    }
    
    /// After frame is renderer clean render data
    #[profiler::function]
    fn after_render(&mut self, state: &mut crate::app::state::State) {
        let gui = &mut state.gui;
        gui.data_to_render = None;
    }
}
