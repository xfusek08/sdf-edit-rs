mod application;
mod profiler;

use application::{Application, ApplicationConfig};
use log::info;

fn main() {
    env_logger::init();
    
    info!("starting...");
    let config = ApplicationConfig {
        // TODO: Global configuration here
        ..ApplicationConfig::default()
    };
    
    let mut app = {
        profiler_session_begin!("app_create-profile");
        Application::new(config)
    };
    
    profiler_session_begin!("app_run-profile");
    app.run();
}
