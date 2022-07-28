/// This is an implementation of profiling instrumentor
/// Outputting a Json Trace Event format profile file.
/// https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview#heading=h.lenwiilchoxp

#[cfg(feature = "profiling")]
pub mod profiler_internal {
    use log::{info, warn};
    use parking_lot::Mutex;
    use serde::Serialize;
    use std::{
        collections::HashMap,
        process,
        sync::Arc,
        thread::ThreadId,
        time::Instant,
    };
    
    static CURRENT: Mutex<Option<Session>> = Mutex::new(None);
    
    #[derive(Serialize, Clone)]
    struct Event {
        name: &'static str,
        #[serde(rename = "cat")]
        category: &'static str,
        #[serde(rename = "ph")]
        phase: EventPhase,
        #[serde(rename = "ts")]
        timestamp: u128,
        #[serde(rename = "pid")]
        process_id: u32,
        #[serde(rename = "tid")]
        thread_id: u32,
        #[serde(rename = "args")]
        arguments: Option<Arc<dyn erased_serde::Serialize + Send + Sync>>,
    }
    
    #[derive(Serialize, Clone)]
    enum EventPhase {
        #[serde(rename = "B")]
        BeginScope,
        #[serde(rename = "E")]
        EndScope,
    }
    
    #[derive(Clone)]
    pub enum EventCategory {
        Performance,
        Custom(&'static str),
    }
    impl Into<&'static str> for EventCategory {
        fn into(self) -> &'static str {
            match self {
                EventCategory::Performance => "PERF",
                EventCategory::Custom(str) => str,
            }
        }
    }
    
    #[derive(Serialize)]
    struct Session {
        #[serde(skip_serializing)]
        pub name: &'static str,
        #[serde(rename = "traceEvents")]
        pub events: Vec<Event>,
        #[serde(skip_serializing)]
        pub start: Instant,
        #[serde(skip_serializing)]
        pub thread_id_map: HashMap<ThreadId, (u32, &'static str)>,
    }
    
    impl Session {
        pub fn new(name: &'static str) -> Self {
            let mut new_session = Self {
                name,
                events: vec![],
                start: Instant::now(),
                thread_id_map: HashMap::new(),
            };
            new_session.register_thread("Main Thread");
            new_session
        }
        pub fn register_thread(&mut self, name: &'static str) -> (ThreadId, u32) {
            let tid = std::thread::current().id();
            return match self.thread_id_map.get(&tid) {
                Some((id, old_name)) => {
                    warn!("Profiler: Cannot register new thread under name \"{}\" because this thread is already registered as \"{}\"", name, old_name);
                    (tid, *id)
                },
                None => {
                    let id = self.thread_id_map.len() as u32;
                    self.thread_id_map.insert(tid, (id, name));
                    (tid, id)
                },
            }
        }
        pub fn current_thread_id(&mut self) -> (ThreadId, u32) {
            let tid = std::thread::current().id();
            return match self.thread_id_map.get(&tid) {
                Some((id, _)) => (tid, *id),
                None => {
                    warn!("Profiler: Event in unregistered thread. This will be displayed as \"Unnamed Thread\" from now on.\nPlease make sure to register reach new thread before logging any profiling events.");
                    self.register_thread("Unknown thread")
                }
            };
        }
        pub fn unregister_thread(&mut self, thread_id: ThreadId) {
            self.thread_id_map.remove(&thread_id);
        }
    }
    
    pub struct SessionGuard;
    impl SessionGuard {
        pub fn new(name: &'static str) -> Option<SessionGuard> {
            let mut current_guard = CURRENT.lock();
            return match current_guard.as_ref() {
                Some(session) => {
                    warn!(
                        "Profiler: Cannot start session \"{}\" because session \"{}\" is still running",
                        name, session.name
                    );
                    None
                }
                None => {
                    *current_guard = Some(Session::new(name));
                    info!("Profiling session \"{name}\" started");
                    Some(SessionGuard)
                }
            };
        }
    }
    impl Drop for SessionGuard {
        fn drop(&mut self) {
            let mut current_guard = CURRENT.lock();
            if current_guard.is_some() {
                {
                    let session_ref = current_guard.as_ref().unwrap();
                    info!("Profiling session \"{}\" ended", session_ref.name);
                    let str = serde_json::to_string(session_ref).unwrap();
                    info!("{}", str);
                }
                *current_guard = None;
            }
        }
    }
    
    pub struct EventGuard {
        event: Event,
    }
    impl EventGuard {
        pub fn new<T>(name: &'static str, category: EventCategory, arguments: Option<T>) -> Option<EventGuard>
        where
            T: erased_serde::Serialize + Send + Sync + 'static,
        {
            let mut current_ref = CURRENT.lock();
            return match current_ref.as_mut() {
                Some(session) => {
                    let event = Event {
                        name,
                        category: category.clone().into(),
                        phase: EventPhase::BeginScope,
                        timestamp: session.start.elapsed().as_micros(),
                        process_id: process::id(),
                        thread_id: session.current_thread_id().1,
                        arguments: if let Some(args) = arguments {
                            Some(Arc::new(args))
                        } else {
                            None
                        },
                    };
                    session.events.push(event.clone());
                    Some(EventGuard { event })
                }
                None => {
                    warn!(
                        "Profiler: cannot log event \"{}\" because no profiling session is running",
                        name
                    );
                    None
                }
            };
        }
    }
    impl Drop for EventGuard {
        fn drop(&mut self) {
            let mut current_ref = CURRENT.lock();
            match current_ref.as_mut() {
                Some(session) => {
                    session.events.push(Event {
                        phase: EventPhase::EndScope,
                        timestamp: session.start.elapsed().as_micros(),
                        ..self.event.clone()
                    });
                }
                None => {
                    warn!(
                        "Profiler: cannot end event \"{}\" because session no longer exists",
                        self.event.name
                    );
                }
            }
        }
    }
    
    pub struct ThreadGuard {
        thread_id: ThreadId,
    }
    impl ThreadGuard {
        pub fn new(name: &'static str) -> Option<ThreadGuard> {
            let mut current_ref = CURRENT.lock();
            return match current_ref.as_mut() {
                Some(session) => {
                    let (thread_id, _) = session.register_thread(name);
                    Some(ThreadGuard { thread_id })
                }
                None => {
                    warn!("Profiler: cannot register thread \"{}\" because no profiling session is running", name);
                    None
                }
            };
        }
    }
    impl Drop for ThreadGuard {
        fn drop(&mut self) {
            let mut current_ref = CURRENT.lock();
            if let Some(session) = current_ref.as_mut() {
                session.unregister_thread(self.thread_id);
            }
        }
    }
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profiler_session_begin {
    ($a:expr) => {
        let _guard = crate::profiler::profiler_internal::SessionGuard::new($a);
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profiler_session_begin {
    () => {};
    ($a:expr) => {};
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profiler_scope {
    ($a:expr) => {
        let _guard = crate::profiler::profiler_internal::EventGuard::new::<()>($a, crate::profiler::profiler_internal::EventCategory::Performance, None);
    };
    ($a:expr, $b:expr) => {
        let _guard = crate::profiler::profiler_internal::EventGuard::new($a, crate::profiler::profiler_internal::EventCategory::Performance, Some($b));
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profiler_scope {
    () => {};
    ($a:expr) => {};
    ($a:expr, $b:expr) => {};
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profiler_register_thread {
    ($a:expr) => {
        let _guard = crate::profiler::profiler_internal::ThreadGuard::new($a);
    };
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profiler_register_thread {
    () => {};
    ($a:expr) => {};
}
