
use log::warn;
use parking_lot::Mutex;
use std::{collections::{HashMap, VecDeque}, time::Duration};

pub static STATISTICS: Mutex<Option<Statistic>> = Mutex::new(None);

/// This function will start the statistics collection.
pub fn init_statistics() {
    let mut statistics = STATISTICS.lock();
    if statistics.is_some() {
        warn!("Statistics already initialized");
    } else {
        *statistics = Some(Statistic {
            map: HashMap::new(),
            filter: "".to_owned(),
        });
    }
}

pub struct Statistic {
    map: HashMap<&'static str, StatisticRecord>,
    pub filter: String,
}

impl Statistic {
    pub fn pinned(&self) -> impl Iterator<Item = (&'static str, &StatisticRecord)> {
        self.filtered().filter(|(_, record)| record.pinned)
    }
    
    pub fn unpinned(&self) -> impl Iterator<Item = (&'static str, &StatisticRecord)> {
        self.filtered().filter(|(_, record)| !record.pinned)
    }
    
    pub fn filtered(&self) -> impl Iterator<Item = (&'static str, &StatisticRecord)> {
        let filter = self.filter.to_lowercase();
        self.map.iter()
            .filter(move |(name, _)| {
                if filter.is_empty() {
                    true
                } else {
                    name.to_lowercase().contains(filter.as_str())
                }
            })
            .map(|(name, record)| (*name, record))
    }
    
    pub fn pin(&mut self, name: &'static str) {
        if let Some(record) = self.map.get_mut(name) {
            record.pinned = true;
        }
    }
    
    pub fn unpin(&mut self, name: &'static str) {
        if let Some(record) = self.map.get_mut(name) {
            record.pinned = false;
        }
    }
}

const HISTORY_LENGTH: u32 = 100;

#[derive(Clone)]
pub struct StatisticRecord {
    pub pinned: bool,
    pub count: u32,
    pub total_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
    pub history: CircularBuffer<Duration>,
}

impl StatisticRecord {
    pub fn new(pinned: bool) -> Self {
        StatisticRecord {
            count: 0,
            pinned,
            total_time: Duration::from_secs(0),
            max_time: Duration::from_secs(0),
            min_time: Duration::from_secs(u64::MAX),
            history: CircularBuffer::new(HISTORY_LENGTH as usize),
        }
    }

    pub fn add(&mut self, duration: Duration) {
        self.count += 1;
        self.total_time += duration;
        self.max_time = self.max_time.max(duration);
        self.min_time = self.min_time.min(duration);
        self.history.push(duration);
    }

    pub fn average(&self) -> Duration {
        self.total_time / self.count
    }

    pub fn history(&self) -> impl Iterator<Item = &Duration> {
        self.history.iter()
    }
    
    pub fn latest(&self) -> Duration {
        let latest = self.history.buffer.back();
        if let Some(latest) = latest {
            return latest.clone();
        }
        Duration::from_secs(0)
    }
}

/// This is a guard that will be created when a scope is entered and on drop it will enter the measured time into the statistic.
pub struct TimedScope {
    name: &'static str,
    start: std::time::Instant,
    pinned: bool,
}

impl TimedScope {
    pub fn new(name: &'static str, pinned: bool) -> Self {
        TimedScope {
            name,
            start: std::time::Instant::now(),
            pinned,
        }
    }
}

impl Drop for TimedScope {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let mut statistics = STATISTICS.lock();
        if let Some(current) = statistics.as_mut() {
            let record = current.map.entry(self.name)
                .or_insert_with(|| StatisticRecord::new(self.pinned));
            record.add(duration);
        }
    }
}

#[derive(Clone)]
pub struct CircularBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, element: T) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(element);
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }
}
