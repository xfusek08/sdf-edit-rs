
mod app;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

use app::application::{Application, ApplicationConfig, UpdateResult};
use app::clock::Clock;

fn main() {
    env_logger::init();
    profiler::session_begin! ("sdf-editor-app");
    info!("Starting...");
    pollster::block_on(run());
    info!("Exiting");
}


#[profiler::function]
pub async fn run() {
        
    let mut event_loop = profiler::call!(EventLoop::new());
    let window = profiler::call!(winit::window::Window::new(&event_loop).unwrap());
    
    let mut app = Application::new(&window, ApplicationConfig::default()).await;
    
    
    { profiler::scope!("event_loop");
        let mut wall_clock = Clock::now(30);
        event_loop.run_return(move |event, _, control_flow| {
            if wall_clock.tick() {
                app.update(wall_clock.current_tick());
            }
            *control_flow = ControlFlow::WaitUntil(wall_clock.next_scheduled_tick().clone());
            
            match event {
                Event::WindowEvent { event: WindowEvent::Resized(size), ..} =>
                    app.resize(size),
                Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { new_inner_size, .. }, .. } =>
                    app.resize(*new_inner_size),
                Event::WindowEvent { event: WindowEvent::CloseRequested, ..} =>
                    *control_flow = ControlFlow::Exit,
                Event::WindowEvent { event, .. } => {
                    match app.input(&event) {
                        UpdateResult::Exit => *control_flow = ControlFlow::Exit,
                        UpdateResult::Redraw => window.request_redraw(),
                        _ => {},
                    }
                }
                // draw frame with renderer when window requests a redraw
                Event::RedrawRequested(_) => {
                    app.render();
                },
                _ => {}
            }
        });
    }
}
