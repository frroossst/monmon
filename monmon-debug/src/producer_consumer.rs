use std::{cell::UnsafeCell, sync::{Arc, Mutex}, thread};

use colored::Colorize;
use monmon_impl::monitors::BinarySemaphore;

use crate::config::{Config, RaceCondition};

#[derive(Debug)]
pub struct UnsafeSharedBuffer {
    value: UnsafeCell<i64>,
}

unsafe impl Send for UnsafeSharedBuffer {}
unsafe impl Sync for UnsafeSharedBuffer {}

impl Default for UnsafeSharedBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsafeSharedBuffer {
    pub fn new() -> Self {
        UnsafeSharedBuffer {
            value: UnsafeCell::new(0),
        }
    }

    pub fn get(&self) -> i64 {
        unsafe { *self.value.get() }
    }

    pub fn produce(&self) {
        unsafe {
            let current_value = *self.value.get();
            crate::work::do_something();
            *self.value.get() = current_value + 1;
            crate::work::do_something();
        }
    }

    pub fn consume(&self) {
        unsafe {
            let current_value = *self.value.get();
            crate::work::do_something();
            *self.value.get() = current_value - 1;
            crate::work::do_something();
        }
    }

}

pub fn unsafe_multi_threaded_buffer(config: Arc<Config>) -> Box<RaceCondition<i64>> {
    println!(
        "{}",
        "unsafe_multi_threaded_buffer()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let buffer = Arc::new(UnsafeSharedBuffer::default());
    let mut handles = vec![];

    for _ in 0..config.num_producer {
        let accum = buffer.clone();
        let config = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                accum.produce();
            }
        });
        handles.push(handle);
    }

    for _ in 0..config.num_producer {
        let accum = buffer.clone();
        let config = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                accum.consume();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }


    let expected: i64 = 0;
    let race = RaceCondition::new(expected, buffer.get());
    Box::new(race)
}

pub fn stdlib_mutex_multi_threaded_buffer(config: Arc<Config>) -> Box<RaceCondition<i64>> {
    println!(
        "{}",
        "stdlib_mutex_multi_threaded_buffer()"
            .to_string()
            .bright_cyan()
            .italic()
    );

    let buffer = Arc::new(UnsafeSharedBuffer::default());
    let mut handles = vec![];

    let monitor = Arc::new(Mutex::new(()));

    for _ in 0..config.num_producer {
        let accum = buffer.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                // unsafe {
                {
                    let _guard = monitor.lock().unwrap();
                    accum.produce();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    for _ in 0..config.num_producer {
        let accum = buffer.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                // unsafe {
                {
                    let _guard = monitor.lock().unwrap();
                    accum.consume();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let expected: i64 = 0;
    let race = RaceCondition::new(expected, buffer.get());
    Box::new(race)
}

pub fn binary_semaphore_multi_threaded_buffer(config: Arc<Config>) -> Box<RaceCondition<i64>> {
    println!(
        "{}",
        "binary_semaphore_multi_threaded_buffer()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedBuffer::default());
    let mut handles = vec![];

    let monitor = Arc::new(BinarySemaphore::new(1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                {
                    // critical section
                    monitor.P_wait();
                    accum.produce();
                    monitor.V_signal();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                {
                    // critical section
                    monitor.P_wait();
                    accum.consume();
                    monitor.V_signal();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = 0;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn happylock_multi_threaded_buffer(config: Arc<Config>) -> Box<RaceCondition<i64>> {
    println!(
        "{}",
        "happylock_multi_threaded_buffer()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedBuffer::default());
    let mut handles = vec![];

    let monitor = Arc::new(happylock::Mutex::new(()));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                {
                    // critical section
                    let key = happylock::ThreadKey::get().unwrap();
                    let _unused = monitor.lock(key);
                    accum.produce();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                {
                    // critical section
                    let key = happylock::ThreadKey::get().unwrap();
                    let _unused = monitor.lock(key);
                    accum.consume();
                } // end critical section
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = 0;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}


