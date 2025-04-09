use std::sync::{Arc, Mutex};
use std::thread;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use colored::Colorize;

use monmon_debug::config::{Config, ConfigKind};
use monmon_impl::monitors::{MonitorKind, Monitor, SharedMonitor};


enum RaceKind {
    Unsafe,
    StdlibMutex,
    Semaphore,
    HappyLock,
    SemaphoreMonitor,
}


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

fn unsafe_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "unsafe_multi_threaded_accumulator()".to_string().cyan()); 
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];


    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {

                    { // critical section
                        accum.increment();
                    } // end critical section
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

fn stdblib_mutex_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "stdlib_mutex_multi_threaded_accumulator()".to_string().cyan());
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];

    let monitor = Arc::new(Mutex::new(()));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {

                // unsafe {
                    { // critical section
                        let _unused = monitor.lock().unwrap();
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

fn sem_monitor_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
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
                        accum.increment();
                        m.leave();
                    });
                    
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



fn race(racekind: RaceKind, config: Arc<Config>) {

    let result = match racekind {
        RaceKind::Unsafe => {
            unsafe_multi_threaded_accumulator(config)
        },
        RaceKind::StdlibMutex => {
            stdblib_mutex_multi_threaded_accumulator(config)
        },
        RaceKind::SemaphoreMonitor => {
            sem_monitor_multi_threaded_accumulator(config)
        },
        _ => unimplemented!()
    };

    if result.expected != result.actual {
        print!("{}", "[RACE CONDITION] ".red());
        println!("Expected: {}, Actual: {}", result.expected, result.actual);
        println!("Difference: {}", result.expected - result.actual);
    } else {
        println!("{}", "[NO RACE]".green());
    }
}

fn main() {

    let config = Arc::new(Config::new(ConfigKind::Slow));

    race(RaceKind::Unsafe, config.clone());
    race(RaceKind::StdlibMutex, config.clone());
    

}
