
#[cfg(feature = "enabled")]
mod counters;

#[cfg(feature = "enabled")]
pub use counters::Counters;

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! init {
    () => {
        counters::Counters::init()
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! init {
    () => {};
}


#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! deinit {
    () => {
        counters::Counters::deinit()
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! deinit {
    () => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! register {
    ($a:expr) => {
        counters::Counters::register($a)
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! register {
    ($a:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! with_counters {
    ($a:expr) => {
        counters::Counters::with_counters($a)
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! with_counters {
    ($a:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! sample {
    ($a:expr, $b:expr) => {
        counters::Counters::sample($a, $b)
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! sample {
    ($a:expr, $b:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! clear {
    ($a:expr) => {
        counters::Counters::clear($a)
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! clear {
    ($a:expr) => {};
}

#[cfg(feature = "enabled")]
#[macro_export]
macro_rules! clear_all {
    ($a:expr) => {
        counters::Counters::clear_all()
    };
}
#[cfg(not(feature = "enabled"))]
#[macro_export]
macro_rules! clear_all {
    ($a:expr) => {};
}
