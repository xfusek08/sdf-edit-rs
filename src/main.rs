
// mod app;
mod framework;
mod sdf;
mod demo_app;
mod shape_builder;

use crate::framework::application;

fn main() {
    env_logger::init();
    profiler::session_begin!("sdf-editor-app");
    counters::init!();
    info!("Starting...");
    pollster::block_on(application::run(
        application::ApplicationDescriptor {
            define_renderer: demo_app::define_renderer,
            define_updater:  demo_app::define_updater,
            init_scene:      demo_app::init_scene,
            style_gui:       demo_app::style_gui,
        },
        application::RunParams {
            ..Default::default()
        }
    ));
    counters::deinit!();
    info!("Exiting");
}
