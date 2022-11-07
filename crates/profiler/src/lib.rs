
pub use instrumentation_macro::*;

#[cfg(feature = "enabled")]
pub use instrumentation_code::*;

#[macro_export]
macro_rules! add_file_line {
    ($a:expr) => { concat!($a, " (", file!(), ":", line!() ,")") }
}

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
#[macro_export(local_inner_macros)]
macro_rules! scope {
    ($a:expr) => {
        let _guard = profiler::EventGuard::new::<()>(add_file_line!($a), profiler::EventCategory::Performance, None);
    };
    ($a:expr, $b:expr) => {
        let _guard = profiler::EventGuard::new(add_file_line!($a), profiler::EventCategory::Performance, Some($b));
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


#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! call {
    ($($a:tt)*) => {
        { profiler::scope!(stringify!($($a)*)); $($a)* }
    };
}

#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! call {
    ($($a:tt)*) => { $($a)* };
}
