
#[cfg(feature = "profiling")]
pub mod profiler_session {
    use std::fs;
    use tracing::dispatcher::DefaultGuard;
    pub use tracing_chrome::ChromeLayerBuilder;
    pub use tracing_chrome::FlushGuard;
    pub use tracing_subscriber::prelude::*;
    
    pub fn init(name: &str) -> (FlushGuard, DefaultGuard) {
        let dir_name = "profile";
        fs::create_dir_all(dir_name).expect("Failed to prepare profile directory.");
        
        let (chrome_layer, guard) = ChromeLayerBuilder::new()
            .file(format!("{dir_name}/{name}.json"))
            .build();
            
        let registry_guard = tracing_subscriber::registry()
            .with(chrome_layer)
            .set_default();
        
        return (guard, registry_guard);
    }
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! begin_profiler_session {
    ($a:expr) => {
        use profiler::profiler_session;
        let _guard = profiler_session::init($a);
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! begin_profiler_session {
    () => {};
    ($name:expr) => {};
}
