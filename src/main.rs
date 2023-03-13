
// mod app;
mod framework;
mod sdf;
mod demo_app;
mod shape_builder;

use crate::framework::application;

fn main() {
    log_init!();
    
    profiler::session_begin!("sdf-editor-app");
    
    counters::init!();
    
    info!("Starting...");
    
    pollster::block_on(application::run(
        application::ApplicationDescriptor {
            init_renderer: demo_app::init_renderer,
            init_updater:  demo_app::init_updater,
            init_scene:    demo_app::init_scene,
            style_gui:     demo_app::style_gui,
        },
        application::RunParams {
            ..Default::default()
        }
    ));
    
    counters::deinit!();
    
    info!("Exiting");
}
