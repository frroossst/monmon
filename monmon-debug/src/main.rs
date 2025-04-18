use colored::Colorize;
use monmon_impl::monitors::{MonitorKind, SharedMonitor};
use std::cell::UnsafeCell;
use std::sync::Arc;

use monmon_debug::accumulators::*;
use monmon_debug::config::{Config, ConfigKind};

enum RaceKind {
    Unsafe,
    StdlibMutex,
    BinarySemaphore,
    HappyLock,
    SemaphoreMonitor,
}

fn race(racekind: RaceKind, config: Arc<Config>) {
    let start = std::time::Instant::now();
    let result = match racekind {
        RaceKind::Unsafe => std::hint::black_box(unsafe_multi_threaded_accumulator(config)),
        RaceKind::SemaphoreMonitor => {
            std::hint::black_box(sem_monitor_multi_threaded_accumulator(config))
        }
        RaceKind::StdlibMutex => {
            std::hint::black_box(stdblib_mutex_multi_threaded_accumulator(config))
        }
        RaceKind::BinarySemaphore => {
            std::hint::black_box(binary_semaphore_multi_threaded_accumulator(config))
        }
        RaceKind::HappyLock => std::hint::black_box(happylock_multi_threaded_accumulator(config)),
    };

    print!("{:?}", result);

    let elapsed = start.elapsed().as_millis();
    println!("{}", format!("{} ms", elapsed).yellow());

    println!("==========================");
    println!();
}


fn main() {
    let mut args = std::env::args();
    let _program = args.next().expect("program name expected");

    let mode = args.next().unwrap_or("fast".into());
    let config = match mode.as_str() {
        "slow" => Config::new(ConfigKind::Slow),
        "medium" => Config::new(ConfigKind::Medium),
        _ => Config::new(ConfigKind::Fast),
    };

    println!("{:?}", config);

    let config = Arc::new(config);

    race(RaceKind::Unsafe, config.clone());
    race(RaceKind::SemaphoreMonitor, config.clone());
    race(RaceKind::StdlibMutex, config.clone());
    race(RaceKind::BinarySemaphore, config.clone());
    race(RaceKind::HappyLock, config.clone());
}
