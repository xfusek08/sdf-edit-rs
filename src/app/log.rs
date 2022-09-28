// This is a wrapper implementation for logging to any log command will have time measured by profiler

#[macro_export]
macro_rules! profiled_log {
    ($a:ident, $($b:tt)*) => {
        {
            profiler::scope!(concat!(stringify!($a), "!(", stringify!($($b)*), ")"));
            ::log::$a!($($b)*);
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($a:tt)*) => { profiled_log!(debug, $($a)*); };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($a:tt)*) => { profiled_log!(info, $($a)*); };
}
#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($($a:tt)*) => { profiled_log!(warn, $($a)*); };
}
#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($a:tt)*) => { profiled_log!(error, $($a)*); };
}
