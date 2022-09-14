
use winit_input_helper::WinitInputHelper;
use winit::{
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
    error::OsError,
    platform::run_return::EventLoopExtRunReturn,
    event::Event,
};

use crate::{error, app::updating::UpdateContext, info, warn};

use super::{
    scene::{Scene, components::Deleted},
    rendering::{Renderer, modules::{line_renderer::LineRenderer, gui_renderer::GuiRenderer}},
    updating::{Updater, modules::camera_updater::CameraUpdater},
    clock::Clock, gui::Gui
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

#[derive(Default)]
pub struct ApplicationConfig;

pub async fn run(config: ApplicationConfig) {
    
    let mut event_loop: EventLoop<()> = EventLoop::new();
    let window:         Window        = create_window(&event_loop, config).unwrap();
    
    // Updating system
    let mut updater = Updater::new()
        .with_module::<CameraUpdater>();
    
    // Rendering system
    let mut renderer = Renderer::new(&window).await
        .with_module::<LineRenderer>()
        .with_module::<GuiRenderer>();
    
    // Application state
    let mut input: WinitInputHelper = WinitInputHelper::new(); // Helps with translating window events to remembered input state
    let mut clock: Clock            = Clock::now(30);          // 30 ticks per second
    let mut scene: Scene            = Scene::new();            // contains all that is to be rendered and can be updated
    let mut gui:   Gui              = Gui::new(&event_loop);   // Application gui, capable of rendering and altering scene
    
    // this is hack around input helper to only call input update on window events
    let mut update_scene = false;
    
    // Main loop
    { profiler::scope!("event_loop");
        event_loop.run_return(move |event, _, control_flow| {
            
            // Proces window events
            let mut flow_result_action = ControlFlowResultAction::None;
            match event {
                Event::NewEvents(_) |
                Event::MainEventsCleared |
                Event::WindowEvent { .. } => {
                    profiler::scope!("Processing event");
                    
                    // let gui process window event and when it does not handle it, update scene
                    if let Event::WindowEvent { event, .. } = &event {
                        update_scene = !gui.on_event(&event);
                    }
                    
                    // Let input helper process event to somewhat coherent input state and work with that.
                    if input.update(&event) {
                        if let Some(size) = input.window_resized() {
                            renderer.resize(size, input.scale_factor().unwrap_or(1.0));
                        } else if let Some(scale_factor) = input.scale_factor_changed() {
                            renderer.resize(window.inner_size(), scale_factor);
                        } else if input.quit() {
                            dbg!("Quit");
                            flow_result_action = ControlFlowResultAction::Exit;
                        } else if update_scene {
                            flow_result_action = updater.input(
                                &mut gui,
                                &mut scene,
                                &UpdateContext {
                                    input: &input,
                                    tick: clock.current_tick(),
                                    window: &window,
                                }
                            );
                            update_scene = false;
                            // dbg!(&flow_result_action);
                        }
                    }
                },
                Event::RedrawRequested(_) => {
                    // Render frame when windows requests a redraw not on every update
                    // This is because application could only redraw when there are changes saving CPU time and power.
                    profiler::scope!("Redraw requested");
                    
                    // scene is not changed in prepare (to allow renderer to prepare in parallel)
                    renderer.prepare(&gui, &scene);
                    
                    renderer.render();
                    
                    // TODO: This is possible meant to run in a separate thread alongside the render
                    remove_deleted_entities(&mut scene);
                    renderer.finalize(&mut gui, &mut scene);
                },
                _ => {} // Ignore other events
            }
            
            // Tick clock and update on tick if app is still running
            if clock.tick() {
                // It is time to tick the application
                flow_result_action = flow_result_action.combine(
                    updater.update(
                        &mut gui,
                        &mut scene,
                        &UpdateContext {
                            input: &input,
                            tick: clock.current_tick(),
                            window: &window,
                        }
                    )
                );
                // print!("update | input | render: {} | {} | {}\n", updater.update_cnt, updater.input_cnt, renderer.render_cnt);
            } else {
                // Schedule next tick as a time to wake up in case of idling
                *control_flow = ControlFlow::WaitUntil(clock.next_scheduled_tick().clone())
            };
            
            // Decide on final control flow based on combination of all result actions
            match flow_result_action {
                ControlFlowResultAction::Exit => *control_flow = ControlFlow::Exit,
                ControlFlowResultAction::Redraw => window.request_redraw(),
                _ => {},
            }
        });
    }
}

#[profiler::function]
fn create_window<T>(event_loop: &EventLoop<T>, config: ApplicationConfig) -> Result<Window, OsError> {
    WindowBuilder::new()
        .with_title("Rust Game")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
}

#[profiler::function]
pub fn remove_deleted_entities(scene: &mut Scene) {
    
    // fill buffer with entities to delete
    let mut entities_to_delete = Vec::with_capacity(scene.world.len() as usize);
    for (entity, (Deleted(deleted),)) in scene.world.query::<(&Deleted,)>().iter() {
        if *deleted {
            entities_to_delete.push(entity);
        }
    }
    
    // delete entities
    for entity in entities_to_delete {
        if let Err(_) = scene.world.despawn(entity) {
            error!("Failed to despawn entity {:?}", entity);
        }
    }
    
}
