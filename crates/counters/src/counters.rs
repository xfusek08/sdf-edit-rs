use std::{collections::{HashMap, vec_deque::Iter}, ops::Deref, time::{Instant, Duration}};

use circular_buffer::CircularBuffer;
use parking_lot::{Mutex, MutexGuard};

static STATISTICS: Mutex<Option<Counters>> = Mutex::new(None);

pub struct CounterRecord {
    pub total: f64,
    pub history: CircularBuffer<(Instant, f64)>,
}

pub struct HistoryIterator<'a> {
    up_to: Instant,
    is_first: bool,
    last_time: Instant,
    iter: Iter<'a, (Instant, f64)>,
}

impl<'a> HistoryIterator<'a> {
    pub fn for_record(record: &'a CounterRecord, duration: Duration) -> Self {
        Self {
            up_to: Instant::now() - duration,
            last_time: Instant::now(),
            is_first: true,
            iter: record.history.iter(),
        }
    }
}

impl<'a> Iterator for HistoryIterator<'a> {
    type Item = (Duration, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((time, value)) = self.iter.next() else {
            if self.is_first {
                self.is_first = false;
                return Some((self.last_time - Instant::now(), 0.0));
            }
            return None;
        };
        if *time < self.up_to {
            if self.is_first {
                self.is_first = false;
                return Some((self.last_time - *time, *value));
            }
            return None;
        }
        Some((self.last_time - *time, *value))
    }
}

impl CounterRecord {
    pub fn new() -> Self {
        Self {
            total: 0.0,
            history: CircularBuffer::new(1000),
        }
    }
    
    pub fn sample(&mut self, value: f64) {
        self.total += value;
        self.history.push_front((Instant::now(), value));
    }
    
    pub fn clear(&mut self) {
        self.history.clear();
    }
    
    /// Returns the latest sample
    pub fn latest_sample(&self) -> Option<&(Instant, f64)> {
        self.history.first()
    }
    
    pub fn get_latest_value(&self) -> f64 {
        let Some((_, value)) = self.latest_sample() else {
            return 0.0;
        };
        *value
    }
    
    pub fn iter_past(&self, duration: Duration) -> HistoryIterator {
        HistoryIterator::for_record(self, duration)
    }
    
    /// Returns the duration of the last sample
    pub fn duration_of_last_sample(&self) -> std::time::Duration {
        let Some(( last_time, _ )) = self.latest_sample() else {
            return std::time::Duration::ZERO;
        };
        let Some((previous_time, _)) = self.history.nth_from_front(1) else {
            return Instant::now() - *last_time; // This is the first sample
        };
        *last_time - *previous_time // This is the duration between the last two samples
    }
    
    /// Returns the average duration of the last `duration`
    pub fn average_duration_past(&self, samples: usize) -> std::time::Duration {
        let Some(( last_time, _ )) = self.latest_sample() else {
            return std::time::Duration::ZERO;
        };
        
        let samples = samples.min(self.history.len() - 1);
        
        if samples > 0 {
            let (sampled_time, _)  = self.history.nth_from_front(samples).unwrap();
            (*last_time - *sampled_time) / samples as u32
        } else {
            std::time::Duration::ZERO
        }
    }
    
    /// Returns the average value of the last `duration`
    pub fn average_past_value(&self, duration: Duration) -> f64 {
        let (sum, count) = self.iter_past(duration)
            .fold((0.0, 0), |(sum, count), (_, value)| (sum + value, count + 1));
        sum / count as f64
    }
    
    #[inline]
    pub fn average_past_value_seconds(&self, seconds: f64) -> f64 {
        self.average_past_value(std::time::Duration::from_secs_f64(seconds))
    }
    
    #[inline]
    pub fn average_past_value_second(&self) -> f64 {
        self.average_past_value_seconds(1.0)
    }
    
    pub fn sum_past_values(&self, duration: Duration) -> f64 {
        self.iter_past(duration)
            .fold(0.0, |sum, (_, value)| sum + value)
    }
    
    #[inline]
    pub fn sum_past_values_seconds(&self, seconds: f64) -> f64 {
        self.sum_past_values(std::time::Duration::from_secs_f64(seconds))
    }
    
    #[inline]
    pub fn sum_past_values_second(&self) -> f64 {
        self.sum_past_values_seconds(1.0)
    }
    
}



pub struct Counters {
    map: HashMap<&'static str, CounterRecord>,
}

impl Counters {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    pub fn get_latest_value(&self, name: &'static str) -> f64 {
        let Some(record) = self.map.get(name) else {
            return 0.0;
        };
        record.get_latest_value()
    }
    
    pub fn get_total(&self, name: &'static str) -> f64 {
        let Some(record) = self.map.get(name) else {
            return 0.0;
        };
        record.total
    }
}

impl Deref for Counters {
    type Target = HashMap<&'static str, CounterRecord>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Counters {
    pub fn init() {
        *STATISTICS.lock() = Some(Self::new());
    }
    
    pub fn deinit() {
        *STATISTICS.lock() = None;
    }
    
    pub fn lock() -> MutexGuard<'static, Option<Counters>> {
        STATISTICS.lock()
    }
    
    pub fn with_counters<R>(func: impl FnOnce(&mut Counters) -> R) -> R {
        let mut lock = Self::lock();
        let counters = lock.as_mut().expect("Counters not initialized");
        func(counters)
    }
    
    pub fn register(name: &'static str) {
        Self::with_counters(|counters| {
            if counters.map.contains_key(name) {
                log::warn!("Counter '{}' already registered", name);
                return;
            }
            counters.map.insert(name, CounterRecord::new());
        });
    }
    
    pub fn sample(name: &'static str, value: f64) {
        Self::with_counters(|counters| {
            let Some(record) = counters.map.get_mut(name) else {
                log::warn!("Counter '{}' not registered", name);
                return;
            };
            record.sample(value);
        });
    }
    
    pub fn clear(name : &'static str) {
        Self::with_counters(|counters| {
            let Some(record) = counters.map.get_mut(name) else {
                log::warn!("Counter '{}' not registered", name);
                return;
            };
            record.clear();
        });
    }
    
    pub fn clear_all() {
        Self::with_counters(|counters| {
            for record in counters.map.values_mut() {
                record.clear();
            }
        });
    }
}
