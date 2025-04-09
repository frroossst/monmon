use std::sync::{Arc, Mutex};
use std::thread;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use colored::Colorize;

use monmon_debug::config::Config;
use monmon_impl::monitors::{MonitorKind, Monitor, SharedMonitor};

#[derive(Debug)]
struct RaceCondition {
    expected: usize,
    actual: usize,
}

impl RaceCondition {
    fn new(expected: usize, actual: usize) -> Self {
        RaceCondition { expected, actual }
    }
}

#[derive(Debug)]
struct UnsafeSharedAccumulator {
    value: UnsafeCell<usize>,
}

unsafe impl Send for UnsafeSharedAccumulator {}
unsafe impl Sync for UnsafeSharedAccumulator {}

impl UnsafeSharedAccumulator {
    fn new() -> Self {
        UnsafeSharedAccumulator {
            value: UnsafeCell::new(0),
        }
    }

    fn get(&self) -> usize {
        unsafe { *self.value.get() }
    }

    fn increment(&self) {
        unsafe {
            let current_value = *self.value.get();
            *self.value.get() = current_value + 1;
        }
    }
}

fn monitor_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "monitor_multi_threaded_accumulator()".to_string().cyan());
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];

    let monitor = Arc::new(SharedMonitor::new(MonitorKind::Semaphore, 1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {

                // unsafe {
                    { // critical section
                    monitor.with_monitor(|m| {
                        m.enter();
                        dbg!("monitor says ayo or rather mayo!");
                        m.leave();
                    });
                    
                    // Increment counter inside the critical section
                    accum.increment();
                    } // end critical section
                // }
            }
        });
        handles.push(handle);
    }

    // Join all producer threads
    for handle in handles {
        handle.join().unwrap();
    }

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected,  counter.get());
    Box::new(race)
}

fn main() {

    let config = Arc::new(Config {
        num_producer: 10,
        per_producer: 1000,
        // Add any other required fields from Config struct
    });

    // Run the test and get the race condition result
    let race_result = monitor_multi_threaded_accumulator(config);
    
    println!("Expected sum: {}", race_result.expected);
    println!("Actual sum: {}", race_result.actual);
    
    // Check if a race condition occurred
    if race_result.expected == race_result.actual {
        println!("{}", "No race condition detected! The monitor is working correctly.".green());
    } else {
        println!("{}", "Race condition detected! The monitor failed to protect the shared resource.".red());
        println!("Difference: {} values missing", race_result.expected - race_result.actual);
    }

}
