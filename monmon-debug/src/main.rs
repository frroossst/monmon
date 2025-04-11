use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use colored::Colorize;

use monmon_debug::config::{Config, ConfigKind};
use monmon_impl::monitors::{BinarySemaphore, MonitorKind, SharedMonitor};



fn do_something() {
    // either randomly sleep, busy wait, or do nothing
    let mut rng = rand::rng();
    let random_number = rng.random_range(0..3);
    match random_number {
        0 => {
            let sleep_duration = rng.random_range(1..50);
            thread::sleep(std::time::Duration::from_millis(sleep_duration));
        }
        1 => {
            let busy_wait_duration = rng.random_range(1..50);
            let start_time = std::time::Instant::now();
            while start_time.elapsed().as_millis() < busy_wait_duration as u128 {}
        }
        _ => {
            // Do nothing
        }
    }
}

enum RaceKind {
    Unsafe,
    StdlibMutex,
    BinarySemaphore,
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
            do_something();
            *self.value.get() = current_value + 1;
            do_something();
        }
    }
}

fn unsafe_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "unsafe_multi_threaded_accumulator()".to_string().bright_cyan().italic()); 
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
    println!("{}", "stdlib_mutex_multi_threaded_accumulator()".to_string().bright_cyan().italic());
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
    println!("{}", "monitor_multi_threaded_accumulator()".to_string().bright_cyan().italic());
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];

    let monitor = Arc::new(SharedMonitor::new(MonitorKind::Semaphore, 1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                { // critical section
                    monitor.enter();
                    accum.increment();
                    monitor.leave();
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

fn binary_semaphore_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "binary_semaphore_multi_threaded_accumulator()".to_string().bright_cyan().italic());
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];

    let monitor = Arc::new(BinarySemaphore::new(1));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                { // critical section
                    monitor.P_wait();
                    accum.increment();
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

    let expected = config.num_producer * config.per_producer;
    let race = RaceCondition::new(expected,  counter.get());
    Box::new(race)
}

fn happylock_multi_threaded_accumulator(config: Arc<Config>) -> Box<RaceCondition> {
    println!("{}", "happylock_multi_threaded_accumulator()".to_string().bright_cyan().italic());
    let counter = Arc::new(UnsafeSharedAccumulator::new());
    let mut handles = vec![];

    let monitor = Arc::new(happylock::Mutex::new(()));

    for _ in 0..config.num_producer {
        let accum = counter.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let handle = thread::spawn(move || {
            for _ in 0..config.per_producer {
                { // critical section
                    let key = happylock::ThreadKey::get().unwrap();
                    let _unused = monitor.lock(key);
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


fn race(racekind: RaceKind, config: Arc<Config>) {

    let start = std::time::Instant::now();
    let result = match racekind {
        RaceKind::Unsafe => {
            std::hint::black_box(unsafe_multi_threaded_accumulator(config))
        },
        RaceKind::SemaphoreMonitor => {
            std::hint::black_box(sem_monitor_multi_threaded_accumulator(config))
        },
        RaceKind::StdlibMutex => {
            std::hint::black_box(stdblib_mutex_multi_threaded_accumulator(config))
        },
        RaceKind::BinarySemaphore => {
            std::hint::black_box(binary_semaphore_multi_threaded_accumulator(config))
        },
        RaceKind::HappyLock => {
            std::hint::black_box(happylock_multi_threaded_accumulator(config))
        },
    };

    let elapsed = start.elapsed().as_millis();

    if result.expected != result.actual {
        println!("{}", "[RACE CONDITION] ".red().bold().blink());
        println!("Expected: {}, Actual: {}", result.expected, result.actual);
        println!("Missing items: {}", format!("{}", result.expected - result.actual).bright_white().italic());
        println!("{}", format!("{} ms", elapsed).yellow());
    } else {
        println!("{}", "[NO RACE]".bright_green().bold());
        println!("{}", format!("{} ms", elapsed).yellow());
    }

    println!("==========================");
    println!();
}

fn main() {
    let mut args = std::env::args();
    let _program = args.next().expect("program name");

    let mode = args.next().unwrap_or("fast".into());
    let config = match mode.as_str() {
        "slow" => {
            Config::new(ConfigKind::Slow)
        },
        "medium" => {
            Config::new(ConfigKind::Medium)
        },
        _ => {
            Config::new(ConfigKind::Fast)
        }
    };

    println!("{:?}", config);

    let config = Arc::new(config);

    race(RaceKind::Unsafe, config.clone());
    race(RaceKind::SemaphoreMonitor, config.clone());
    race(RaceKind::StdlibMutex, config.clone());
    race(RaceKind::BinarySemaphore, config.clone());
    race(RaceKind::HappyLock, config.clone());

}
