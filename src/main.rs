mod renderer;
mod data;
mod application;
mod log;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

use application::{Application, ApplicationConfig, UpdateResult};

fn main() {
    env_logger::init();
    profiler::session_begin!("sdf-editor-app");
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
        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                
                Event::WindowEvent { event: WindowEvent::Resized(size), ..} => {
                    app.resize(size);
                },
                
                Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { new_inner_size, .. }, .. } => {
                    app.resize(*new_inner_size);
                },
                
                // exit update loop on close window event
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
                    match app.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => app.resize(window.inner_size()),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => error!("{:?}", e),
                    }
                },
                
                _ => {}
            }
        });
    }
}
