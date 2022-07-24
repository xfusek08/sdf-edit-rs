mod application;
mod profiler;

use application::{Application, ApplicationConfig};

fn main() {
    
    let mut app = {
        begin_profiler_session!("app_create-profile");
        Application::new(ApplicationConfig {
            // TODO: Global configuration here
            ..ApplicationConfig::default()
        })
    };
    
    begin_profiler_session!("app_run-profile");
    app.run();
}
