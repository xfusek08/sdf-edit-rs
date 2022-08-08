
use std::time::{Instant, Duration};

use crate::info;

#[derive(Clone, Debug)]
pub struct Tick {
    pub order: u64,
    pub time: Instant,
    pub delta: Duration,
}

pub struct Clock {
    update_time_window: Duration,
    next_tick_scheduled: Instant,
    current_tick: Tick,
    elapsed_seconds: f32,
    tick_counter: u32,
}

impl Clock {
    pub fn now(tick_per_seconds: u64) -> Self {
        Self {
            update_time_window: Duration::from_secs_f64(1.0 / (tick_per_seconds as f64)),
            next_tick_scheduled: Instant::now(),
            current_tick: Tick {
                order: 0,
                time: Instant::now(),
                delta: Duration::ZERO,
            },
            elapsed_seconds: 0.0,
            tick_counter: 0,
        }
    }
    
    pub fn tick(&mut self) -> bool {
        let time = Instant::now();
        if self.next_tick_scheduled <= time {
            let time_difference = time - self.next_tick_scheduled;
            self.current_tick.order = self.current_tick.order + 1;
            self.current_tick.delta = time - self.current_tick.time;
            self.current_tick.time = time;
            self.next_tick_scheduled = time + self.update_time_window - time_difference;
            
            self.elapsed_seconds += self.current_tick.delta.as_secs_f32();
            self.tick_counter += 1;
            if self.elapsed_seconds > 1.0 {
                info!("Ticks per second: {}", self.tick_counter);
                self.elapsed_seconds -= 1.0;
                self.tick_counter = 0;
            }
            return true;
        }
        false
    }
    
    pub fn current_tick(&self) -> &Tick {
        &self.current_tick
    }
    
    pub fn next_scheduled_tick(&self) -> &Instant {
        &self.next_tick_scheduled
    }
    
}
