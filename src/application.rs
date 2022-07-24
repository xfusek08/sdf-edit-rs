use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application;

// static
impl Application {
    
    /// Creates new application
    #[profiling::function]
    pub fn new(_config: ApplicationConfig) -> Self {
        dbg!("Creating application");
        sleep(Duration::from_secs(1));
        return Self;
    }
}

// public
impl Application {
    
    /// Runs the main loop
    #[profiling::function]
    pub fn run(&mut self) {
        for a in 0..10 {
            profiling::scope!("frame");
            {
                profiling::scope!("dbg!");
                dbg!(a);
            }
            {
                profiling::scope!("sleep");
                sleep(Duration::from_millis(100));
            }
        }
    }
    
}
