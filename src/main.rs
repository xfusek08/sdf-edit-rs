mod application;
mod log;
mod renderer;

use application::{Application, ApplicationConfig};

fn main() {
    profiler::session_begin!("sdf-editor-app");
    
    profiler::call!(env_logger::init());
    
    info!("Starting...");

    let config = ApplicationConfig {
        // TODO: Global configuration here
        ..ApplicationConfig::default()
    };
    
    let mut app = Application::new(config);
    
    pollster::block_on(app.run());
    
    info!("Exiting");
}
