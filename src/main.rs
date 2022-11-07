use crate::app::application::{self, ApplicationConfig};

mod app;
mod framework;
mod sdf;
mod demo_app;

fn main() {
    env_logger::init();
    profiler::session_begin! ("sdf-editor-app");
    info!("Starting...");
    let config = ApplicationConfig::default(); // TODO: load config from file/arguments
    pollster::block_on(application::run(config));
    info!("Exiting");
}
