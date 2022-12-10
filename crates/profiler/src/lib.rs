
pub use instrumentation_macro::*;

#[cfg(feature = "stats")]
pub use runtime_stats::*;

#[cfg(feature = "json_trace")]
pub use json_trace::*;

#[macro_export]
macro_rules! add_file_line {
    ($a:expr) => { concat!($a, " (", file!(), ":", line!() ,")") }
}

#[macro_export]
macro_rules! session_begin {
    ($a:expr) => {
        #[cfg(feature = "json_trace")]
        let _guard = profiler::SessionGuard::new($a);
        #[cfg(feature = "stats")]
        profiler::init_statistics();
    };
}

#[macro_export(local_inner_macros)]
macro_rules! scope {
    ($a:expr) => {
        #[cfg(feature = "json_trace")]
        let _guard = profiler::EventGuard::new::<()>(add_file_line!($a), profiler::EventCategory::Performance, None);
        
        #[cfg(feature = "stats")]
        let _stat_guar = profiler::TimedScope::new(add_file_line!($a), false);
    };
    ($a:expr, pinned) => {
        #[cfg(feature = "json_trace")]
        let _guard = profiler::EventGuard::new::<()>(add_file_line!($a), profiler::EventCategory::Performance, None);
        
        #[cfg(feature = "stats")]
        let _stat_guar = profiler::TimedScope::new(add_file_line!($a), true);
    };
}

#[macro_export]
macro_rules! register_thread {
    ($a:expr) => {
        #[cfg(feature = "json_trace")]
        let _guard = profiler::ThreadGuard::new($a);
    };
}

#[macro_export]
macro_rules! call {
    ($($a:tt)*) => {
        {
            #[cfg(feature = "json_trace")]
            profiler::scope!(stringify!($($a)*));
            $($a)*
        }
    };
}
