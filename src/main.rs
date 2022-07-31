mod application;
mod log;
mod renderer;

use application::{Application, ApplicationConfig};

fn main() {
    env_logger::init();
    
    profiler::session_begin!("sdf-editor-app");
    
    info!("Starting...");

    let config = ApplicationConfig {
        // TODO: Global configuration here
        ..ApplicationConfig::default()
    };
    
    let mut app = Application::new(config);
    
    pollster::block_on(app.run());
    
    info!("Exiting");
}
