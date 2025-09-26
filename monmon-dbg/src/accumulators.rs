use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex},
    thread,
};

use colored::Colorize;
use monmon_impl::{
    critical_section, futex_monitor::FutexMonitor, monitor_trait::Monitor,
    semaphore::BinarySemaphore, semaphore_monitor::SemaphoreMonitor,
};

use crate::config::{Config, RaceCondition};

#[derive(Debug)]
pub struct UnsafeSharedAccumulator {
    value: UnsafeCell<usize>,
}

// SAFETY: We **want** this to have a data race for demonstration purposes
unsafe impl Sync for UnsafeSharedAccumulator {}

impl Default for UnsafeSharedAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsafeSharedAccumulator {
    pub fn new() -> Self {
        UnsafeSharedAccumulator {
            value: UnsafeCell::new(0),
        }
    }

    pub fn get(&self) -> usize {
        // SAFETY: We are allowing unsynchronized access to the internal value
        // for demonstration purposes
        unsafe { *self.value.get() }
    }

    pub fn increment(&self) {
        // SAFETY: We are allowing unsynchronized access to the internal value
        // for demonstration purposes
        unsafe {
            let current_value = *self.value.get();
            crate::work::do_something();
            *self.value.get() = current_value + 1;
            crate::work::do_something();
        }
    }
}

pub fn unsafe_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "unsafe_multi_threaded_accumulator()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedAccumulator::default());
    let mut handles = vec![];

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                critical_section!({
                    accum.increment();
                });
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn stdblib_mutex_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "stdlib_mutex_multi_threaded_accumulator()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedAccumulator::default());
    let mut handles = vec![];

    let monitor = Arc::new(Mutex::new(()));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                critical_section!({
                    let _unused = monitor.lock().unwrap();
                    accum.increment();
                })
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn sem_monitor_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "sem_monitor_multi_threaded_accumulator()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedAccumulator::default());
    let mut handles = vec![];

    let monitor = Arc::new(SemaphoreMonitor::new(1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                critical_section!({
                    let _guard = monitor.lock();
                    // monitor.enter();
                    accum.increment();
                    // monitor.leave();
                })
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn binary_semaphore_multi_threaded_accumulator(
    config: Arc<Config>,
) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "binary_semaphore_multi_threaded_accumulator()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedAccumulator::default());
    let mut handles = vec![];

    let monitor = Arc::new(BinarySemaphore::new(1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                critical_section!({
                    monitor.P_wait();
                    accum.increment();
                    monitor.V_signal();
                })
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn futex_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "futex_multi_threaded_accumulator()"
            .to_string()
            .bright_cyan()
            .italic()
    );
    let counter = Arc::new(UnsafeSharedAccumulator::default());
    let mut handles = vec![];

    let monitor = Arc::new(FutexMonitor::new(1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                critical_section!({
                    monitor.enter();
                    accum.increment();
                    monitor.leave();
                })
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected, counter.get());
    Box::new(race)
}

pub fn ipc_monitor_multi_threaded_accumulator(_config: Arc<Config>) -> Box<RaceCondition<usize>> {
    println!(
        "{}",
        "ipc_monitor_multi_threaded_accumulator"
            .to_string()
            .bright_cyan()
            .italic()
    );
    unimplemented!();
}
