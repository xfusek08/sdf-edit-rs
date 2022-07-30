
pub use instrumentation_macro::*;

#[cfg(feature = "enabled")]
pub use instrumentation_code::*;

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! session_begin {
    ($a:expr) => {
        let _guard = profiler::SessionGuard::new($a);
    };
}

#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! session_begin {
    () => {};
    ($a:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! scope {
    ($a:expr) => {
        let _guard = profiler::EventGuard::new::<()>($a, profiler::EventCategory::Performance, None);
    };
    ($a:expr, $b:expr) => {
        let _guard = profiler::EventGuard::new($a, profiler::EventCategory::Performance, Some($b));
    };
}

#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! scope {
    () => {};
    ($a:expr) => {};
    ($a:expr, $b:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! register_thread {
    ($a:expr) => {
        let _guard = profiler::ThreadGuard::new($a);
    };
}

#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! register_thread {
    () => {};
    ($a:expr) => {};
}