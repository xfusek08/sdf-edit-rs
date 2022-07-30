use std::thread;

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
    pub fn run(&mut self) {
        let render_thread_handle = thread::spawn(move || {
            profiler::register_thread!("Render Thread");
            Self::render_loop();
        });
        Self::update_loop();
        render_thread_handle.join().unwrap();
    }
    
    #[profiler::function]
    fn update_loop() {
        for a in 0..10 {
            profiler::scope!("update");
            info!("update {a}");
        }
    }
    
    #[profiler::function]
    fn render_loop() {
        for a in 0..10 {
            profiler::scope!("render");
            info!("render {a}");
        }
    }
    
}
