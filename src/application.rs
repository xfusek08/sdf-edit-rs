use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, platform::run_return::EventLoopExtRunReturn,
};

use crate::renderer::{Renderer, self};
use crate::info;

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application;

// static
impl Application {
    
    #[profiler::function]
    pub fn new(_config: ApplicationConfig) -> Self {
        dbg!("Creating application");
        return Self;
    }
}

// public
impl Application {
    
    #[profiler::function]
    pub async fn run(&mut self) {
        
        let mut event_loop = profiler::call!(EventLoop::new());
        let window = profiler::call!(winit::window::Window::new(&event_loop).unwrap());
        let mut renderer = profiler::call!(Renderer::new(&window).await);
        
        { profiler::scope!("event_loop");
            event_loop.run_return(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    
                    // Window resize event
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        ..
                    } => {
                        profiler::scope!("WindowEvent::Resized");
                        renderer.resize(size);
                        profiler::call!(window.request_redraw());
                    }
                    
                    // draw frame with renderer when window requests a redraw
                    Event::RedrawRequested(_) => {
                        renderer.draw();
                    }
                    
                    // exit update loop on close window event
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    
                    _ => {}
                }
            });
        }
    }
}
