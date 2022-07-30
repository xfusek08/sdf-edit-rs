mod application;
mod log;

use application::{Application, ApplicationConfig};
use simple_logger::SimpleLogger;

fn main() {
    profiler::session_begin!("sdf-editor-app");
    
    {
        profiler::scope!("Initializing SimpleLogger");
        SimpleLogger::new().init().unwrap();
    }
    
    info!("Starting...");

    let config = {
        profiler::scope!("Creating application config");
        ApplicationConfig {
            // TODO: Global configuration here
            ..ApplicationConfig::default()
        }
    };
    
    
    let mut app = Application::new(config);
    
    app.run();
    
    info!("Exiting");
}
