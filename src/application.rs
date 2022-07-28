use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::info;

use crate::{profiler_register_thread, profiler_scope};

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application;

// static
impl Application {
    
    pub fn new(_config: ApplicationConfig) -> Self {
        profiler_scope!("new");
        dbg!("Creating application");
        sleep(Duration::from_secs(1));
        return Self;
    }
}

// public
impl Application {
    
    pub fn run(&mut self) {
        profiler_scope!("run");
        let render_thread_handle = thread::spawn(move || {
            profiler_register_thread!("Render Thread");
            Self::render_loop();
        });
        Self::update_loop();
        render_thread_handle.join().unwrap();
    }
    
    fn update_loop() {
        profiler_scope!("update_loop");
        let mut rng = rand::thread_rng();
        for a in 0..3 {
            profiler_scope!("update");
            // sleep(Duration::from_millis(rng.gen_range(0..10)));
            info!("update {a}");
        }
    }
    
    fn render_loop() {
        profiler_scope!("render loop");
        let mut rng = rand::thread_rng();
        for a in 0..3 {
            profiler_scope!("render");
            // sleep(Duration::from_millis(rng.gen_range(0..10)));
            info!("render {a}");
        }
    }
    
}
