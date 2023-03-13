use std::sync::Arc;

use winit_input_helper::WinitInputHelper;
use winit::{
    event::{Event, WindowEvent},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
};

use super::{
    gpu,
    renderer::Renderer,
    clock::Clock,
    gui::Gui,
    camera::SceneWithCamera,
    updater::{
        Updater,
        ResizeContext,
        UpdateContext,
        AfterRenderContext,
        UpdateResultAction,
    },
};

#[derive(Clone, Debug)]
pub struct RunParams {
    pub window_name: &'static str,
    pub window_width: u32,
    pub window_height: u32,
    pub tick_per_second: u32,
}
impl Default for RunParams {
    fn default() -> Self {
        Self {
            window_name: "My App",
            window_width: 1280,
            window_height: 720,
            tick_per_second: 30
        }
    }
}

pub struct Context<'a> {
    pub params: &'a RunParams,
    pub window: &'a Window,
    pub gpu: Arc<gpu::Context>,
}

pub struct ApplicationDescriptor<A, B, C, D> {
    pub init_renderer: A,
    pub init_updater: B,
    pub init_scene: C,
    pub style_gui: D,
}

#[profiler::function]
pub async fn run<S, DR, DU, IS, STG>(app_desc: ApplicationDescriptor<DR, DU, IS, STG>, params: RunParams)
where
    S:          SceneWithCamera + Sized,
    for<'a> DR: FnOnce(&'a Context) -> Renderer<S>, // init_renderer
    DU:         FnOnce(&Context)    -> Updater<S>,  // init_updater
    IS:         FnOnce(&Context)    -> S,           // init_scene
    STG:        FnOnce(egui::Style) -> egui::Style, // style_gui
{
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(params.window_name)
        .with_inner_size(winit::dpi::LogicalSize::new(params.window_width, params.window_height))
        .build(&event_loop).unwrap();
    let gpu = std::sync::Arc::new(gpu::Context::new(&window).await);
    
    let context = Context {
        params: &params,
        window: &window,
        gpu:    gpu.clone(),
    };
    
    // init application specifics
    let mut updater = (app_desc.init_updater)(&context);
    let mut renderer = (app_desc.init_renderer)(&context);
    let mut scene = (app_desc.init_scene)(&context);
    
    let mut gui = Gui::new(&event_loop, app_desc.style_gui);
    
    // Execution control
    let mut input = WinitInputHelper::new();
    let mut clock = Clock::now(params.tick_per_second as u64);
    
    // Main loop
    let mut event_consumed_by_gui = false;
    
    // Register measurements
    counters::register!("frame_counter");
    counters::register!("event_counter");
    counters::register!("update_counter");
    
    event_loop.run_return(move |event, _, control_flow| {
        profiler::scope!("Event incoming");
        counters::sample!("event_counter", 1.0);
        
        let mut flow_result_action = UpdateResultAction::None;
        
        match event {
            Event::NewEvents(_) |
            Event::MainEventsCleared |
            Event::WindowEvent { .. } => {
                profiler::scope!("Processing input event");
                
                // Let gui process window event and when it does not handle it, update scene
                if let Event::WindowEvent { event, .. } = &event {
                    match event {
                        // x11 sends this event when moving cursor on that causes to ignore dragging of egui slides and such
                        WindowEvent::AxisMotion { .. } => {},
                        
                        // rest of the events are handled by egui
                        _ => {
                            profiler::scope!("Processing input event by GUI");
                            event_consumed_by_gui = gui.on_event(&event).consumed;
                        },
                    }
                }
                
                // Let input helper process event to somewhat coherent input state and work with that.
                //   (input.update(..) returns true only on Event::MainEventsCleared hence `update_scene` variable)
                if !input.update(&event) {
                    let input_result = if let Some(size) = input.window_resized() {
                        let scale_factor = input.scale_factor().unwrap_or(1.0);
                        renderer.resize(&size, scale_factor);
                        updater.resize(ResizeContext {
                            gui:   &mut gui,
                            scene: &mut scene,
                            size:  &size,
                            scale_factor,
                        })
                    } else if let Some(scale_factor) = input.scale_factor_changed() {
                        let size = &window.inner_size();
                        renderer.resize(&size, scale_factor);
                        updater.resize(ResizeContext {
                            gui:   &mut gui,
                            scene: &mut scene,
                            size:  &size,
                            scale_factor,
                        })
                    } else if input.close_requested() || input.destroyed() {
                        UpdateResultAction::Exit
                    } else if !event_consumed_by_gui {
                        updater.input(UpdateContext {
                            gui:    &mut gui,
                            scene:  &mut scene,
                            input:  &input,
                            tick:   clock.current_tick(),
                            window: &window,
                        })
                    } else {
                        UpdateResultAction::None
                    };
                    
                    flow_result_action = flow_result_action.combine(input_result);
                }
                
            },
            
            // Render frame when windows requests a redraw not on every update
            // This is because application could only redraw when there are changes saving CPU time and power.
            Event::RedrawRequested(_) => {
                profiler::scope!("Processing redraw request");
                
                renderer.prepare(&gui, &scene);
                renderer.render();
                renderer.finalize();
                
                // this could be run in parallel with render and finalize
                updater.after_render(AfterRenderContext {
                    gui:    &mut gui,
                    scene:  &mut scene,
                });
                
                counters::sample!("frame_counter", 1.0);
                
                // Request redraw after immediately after frame is rendered, to let it run as fast as possible and let vSync to limit FPS by blocking
                flow_result_action = UpdateResultAction::Redraw;
            },
            _ => {} // Ignore other events
        }
        
        // Update application only when is its time to do so
        if clock.tick() {
            updater.update(UpdateContext {
                gui:    &mut gui,
                scene:  &mut scene,
                input:  &input,
                tick:   clock.current_tick(),
                window: &window,
            });
            counters::sample!("update_counter", 1.0);
        } else {
            // Schedule next tick as a time to wake up in case of idling
            *control_flow = ControlFlow::WaitUntil(clock.next_scheduled_tick().clone())
        };
        
        // Decide on final control flow based on combination of all result actions
        match flow_result_action {
            UpdateResultAction::Exit => *control_flow = ControlFlow::Exit,
            UpdateResultAction::Redraw => window.request_redraw(),
            _ => {},
        }
    });
    
}
