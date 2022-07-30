///! this is a wrapper implementation for logging to any log command will have time measured by profiler

#[macro_export]
macro_rules! profiled_log {
    ($a:ident, $b:expr) => {
        {
            profiler::scope!(concat!("Log::", stringify!($a), "()"));
            ::log::$a!($b);
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($a:expr) => { profiled_log!(debug, $a); };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($a:expr) => { profiled_log!(info, $a); };
}
#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($a:expr) => { profiled_log!(warn, $a); };
}
#[macro_export(local_inner_macros)]
macro_rules! error {
    ($a:expr) => { profiled_log!(error, $a); };
}
