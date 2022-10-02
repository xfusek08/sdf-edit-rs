mod framework;
mod demo_app;

use crate::{demo_app::DemoApp, framework::RunParams};

fn main() {
    env_logger::init();
    profiler::session_begin! ("sdf-editor-app");
    framework::run(
        |c| DemoApp::new(c),
        RunParams::default()
    );
}
