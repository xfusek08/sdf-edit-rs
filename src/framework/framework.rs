///! This file contains main implementation of event loop and controlling the application

// glium
use egui_glium::EguiGlium;
use winit_input_helper::WinitInputHelper;
use glium::glutin::{
    self,
    dpi::PhysicalSize,
    event::Event,
    event_loop::{
        EventLoop,
        ControlFlow
    }, platform::run_return::EventLoopExtRunReturn,
};

use super::{
    clock::{Clock, Tick},
};

#[derive(Debug, Clone)]
pub enum ControlFlowResultAction {
    None, Redraw, Exit
}
impl ControlFlowResultAction {
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (ControlFlowResultAction::Exit, _) => ControlFlowResultAction::Exit,
            (_, ControlFlowResultAction::Exit) => ControlFlowResultAction::Exit,
            (ControlFlowResultAction::Redraw, _) => ControlFlowResultAction::Redraw,
            (_, ControlFlowResultAction::Redraw) => ControlFlowResultAction::Redraw,
            _ => ControlFlowResultAction::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub handled: bool,
    pub action: ControlFlowResultAction
}
impl Default for UpdateResult {
    fn default() -> Self {
        Self { handled: false, action: ControlFlowResultAction::None }
    }
}
impl UpdateResult {
    pub fn combine(self, other: Self) -> Self {
        Self {
            handled: self.handled || other.handled,
            action: self.action.combine(other.action)
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunParams {
    pub tick_per_second: u32,
}
impl Default for RunParams {
    fn default() -> Self {
        Self {
            tick_per_second: 30
        }
    }
}

pub struct Context<'a> {
    display: &'a glium::Display,
}

pub trait Application {
    fn input(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult;
    fn update(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult;
    fn render(&mut self, display: &glium::Display, gui: &mut EguiGlium);
    fn resize(&mut self, size: PhysicalSize<u32>, scaling_factor: f64);
    fn exit(&mut self);
    fn style_gui(&mut self, style: egui::Style) -> egui::Style;
    fn gui(&mut self, ctx: &egui::Context);
}

#[profiler::function]
pub fn run<A, F>(create_app: F, param: RunParams)
    where
        A: Application,
        F: FnOnce(&Context) -> A
{
    // create window and graphics context
    let mut event_loop = EventLoop::new();
    
    let cb = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(false)
        .with_srgb(true);
        
    let wb = glutin::window::WindowBuilder::new();
        
    // Application framework context
    let display = profiler::call!(glium::Display::new(wb, cb, &event_loop).unwrap());
    let mut gui = profiler::call!(EguiGlium::new(&display, &event_loop));
    let context = Context { display: &display };
    
    // Execution control
    let mut clock = Clock::now(param.tick_per_second);
    let mut input = winit_input_helper::WinitInputHelper::new();
    
    // Create specific App
    let mut app = create_app(&context);
    
    // App specific initiation
    gui.egui_ctx.set_style(app.style_gui((*gui.egui_ctx.style()).clone()));
    
    // Main loop
    
    event_loop.run_return(move |event, _, control_flow| {
        
        // 1) Set default control flow action
        let mut flow_result_action = ControlFlowResultAction::None;
        
        // 2) Handle input
        if profiler::call!(input.update(&event)) {
            profiler::scope!("WinitInputHelper helper update");
            flow_result_action = flow_result_action.combine(
                if let Some(size) = input.window_resized() {
                    app.resize(size, input.scale_factor().unwrap_or(1.0));
                    ControlFlowResultAction::Redraw
                } else if let Some(scale_factor) = input.scale_factor_changed() {
                    app.resize(display.gl_window().window().inner_size(), scale_factor);
                    ControlFlowResultAction::Redraw
                } else if input.quit() {
                    app.exit();
                    ControlFlowResultAction::Exit
                } else {
                    ControlFlowResultAction::None
                }
            );
        }
        
        // 2) Handle window events
        match event {
            Event::WindowEvent { event, ..} => {
                profiler::scope!("Window event processing");
                if !profiler::call!(gui.on_event(&event)) {
                    profiler::scope!("Passing input to Application");
                    flow_result_action = flow_result_action.combine(
                        app.input(&input, clock.current_tick()).action
                    );
                }
            },
            // Render frame when windows requests a redraw not on every update
            // This is because application could only redraw when there are changes saving CPU time and power.
            Event::RedrawRequested(_) => {
                profiler::scope!("Processing redraw request");
                // application is responsible for rendering gui
                gui.run(&display, |egui_ctx| app.gui(egui_ctx));
                app.render(&display, &mut gui);
            },
            _ => {} // Ignore other events
        };
        
        // 3) Tick clock and update on tick if app is still running
        if clock.tick() {
            profiler::scope!("Processing update on tick");
            app.update(&input, clock.current_tick());
            flow_result_action = ControlFlowResultAction::Redraw;
        }
        
        // 4) Decide on final control flow based on combination of all result actions
        *control_flow = ControlFlow::WaitUntil(clock.next_scheduled_tick().clone());
        match flow_result_action {
            ControlFlowResultAction::Exit => *control_flow = ControlFlow::Exit,
            ControlFlowResultAction::Redraw => display.gl_window().window().request_redraw(),
            _ => {},
        }
    });
}
