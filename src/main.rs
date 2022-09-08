
mod app;

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn, window::WindowBuilder,
};

use app::application::{Application, ApplicationConfig, UpdateResult};
use app::clock::Clock;
use winit_input_helper::WinitInputHelper;

fn main() {
    env_logger::init();
    profiler::session_begin! ("sdf-editor-app");
    info!("Starting...");
    pollster::block_on(run());
    info!("Exiting");
}


#[profiler::function]
pub async fn run() {
        
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_title("sdf-edit-rs")
        .build(&event_loop)
        .unwrap();
    
    let mut app = Application::new(&window, ApplicationConfig::default()).await;
    
    let mut input = WinitInputHelper::new();
    { profiler::scope!("event_loop");
    
        // prepare clock to tick 30 times per seconds
        let mut wall_clock = Clock::now(30);
        
        // run the loop
        event_loop.run_return(move |event, _, control_flow| {
            
            // A little helper macro to react to Update Result from application
            macro_rules! process_update_result {
                ($a:expr) => {
                    match $a {
                        UpdateResult::Exit => *control_flow = ControlFlow::Exit,
                        UpdateResult::Redraw => window.request_redraw(),
                        _ => {},
                    }
                }
            }
            
            // Clock update - performs tick every fraction of the second based on initiation value
            if wall_clock.tick() {
                process_update_result!(app.update(&input, wall_clock.current_tick()));
            }
            *control_flow = ControlFlow::WaitUntil(wall_clock.next_scheduled_tick().clone());
            
            // React to window events
            match event {
                Event::NewEvents(_) |
                Event::MainEventsCleared |
                Event::WindowEvent { .. } => {
                    // Let input helper process event to somewhat coherent input state and work with that.
                    if input.update(&event) {
                        
                        if let Some(size) = input.window_resized() {
                            app.resize(size);
                        } else if let Some(_) = input.scale_factor_changed() {
                            app.resize(window.inner_size());
                        } else if input.quit() {
                            *control_flow = ControlFlow::Exit;
                        } else {
                            process_update_result!(app.input(&input, &wall_clock.current_tick()));
                        }
                    }
                },
                
                // Render frame when windows requests a redraw not on every update
                // This is because application could only redraw when there are changes saving CPU time and power.
                // (When no animation is currently active)
                Event::RedrawRequested(_) => {
                    app.render();
                },
                
                // Ignore other events
                _ => {}
            }
        });
    }
}
