use colored::Colorize;
use std::sync::Arc;

use monmon_debug::accumulators::*;
use monmon_debug::producer_consumer::*;
use monmon_debug::config::{Config, ConfigKind, RaceCondition};

enum RaceKind {
    UnsafeAccum,
    UnsafeBuffer,
    StdlibMutexAccum,
    StdlibMutexBuffer,
    BinarySemaphoreAccum,
    BinarySemaphoreBuffer,
    HappyLockAccum,
    HappyLockBuffer,
    SemaphoreMonitorAccum,
    SemaphoreMonitorBuffer,
}

fn race(racekind: RaceKind, config: Arc<Config>) {
    let start = std::time::Instant::now();

    let usizeRaceCondition: Option<RaceCondition<usize>>;
    let i64RaceCondition: Option<RaceCondition<i64>>;
    
    match racekind {
        RaceKind::UnsafeAccum => {
            let r = std::hint::black_box(unsafe_multi_threaded_accumulator(config));
            usizeRaceCondition = Some(*r);
        }
        RaceKind::UnsafeBuffer => {
            let r = std::hint::black_box(unsafe_multi_threaded_buffer(config));
            i64RaceCondition = Some(*r);
        }
        RaceKind::SemaphoreMonitorAccum => {
            let r = std::hint::black_box(sem_monitor_multi_threaded_accumulator(config));
            usizeRaceCondition = Some(*r);
        }
        RaceKind::SemaphoreMonitorBuffer => {
            let r = std::hint::black_box(sem_monitor_multi_threaded_buffer(config));
            i64RaceCondition = Some(*r);
        }
        RaceKind::StdlibMutexAccum => {
            let r =std::hint::black_box(stdblib_mutex_multi_threaded_accumulator(config));
            usizeRaceCondition = Some(*r);
        }
        RaceKind::StdlibMutexBuffer => {
            let r = std::hint::black_box(stdlib_mutex_multi_threaded_buffer(config));
            i64RaceCondition = Some(*r);
        }
        RaceKind::BinarySemaphoreAccum => {
            let r = std::hint::black_box(binary_semaphore_multi_threaded_accumulator(config));
            usizeRaceCondition = Some(*r);
        }
        RaceKind::BinarySemaphoreBuffer => {
            let r = std::hint::black_box(binary_semaphore_multi_threaded_buffer(config));
            i64RaceCondition = Some(*r);
        }
        RaceKind::HappyLockAccum => {
            let r = std::hint::black_box(happylock_multi_threaded_accumulator(config));
            usizeRaceCondition = Some(*r);
        }
        RaceKind::HappyLockBuffer => {
            let r = std::hint::black_box(happylock_multi_threaded_buffer(config));
            i64RaceCondition = Some(*r);
        }
    };

    if usizeRaceCondition.is_some() {
        let result = usizeRaceCondition.unwrap();
        print!("{:?}", result);
    } else if i64RaceCondition.is_some() {
        let result = i64RaceCondition.unwrap();
        print!("{:?}", result);
    } else {
        unreachable!("both races are None!!!");
    }

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

    let a = stdlib_mutex_multi_threaded_buffer(config.clone());
    println!("{:?}", a);

    race(RaceKind::UnsafeAccum, config.clone());
    race(RaceKind::SemaphoreMonitorAccum, config.clone());
    race(RaceKind::StdlibMutexAccum, config.clone());
    race(RaceKind::BinarySemaphoreAccum, config.clone());
    race(RaceKind::HappyLockAccum, config.clone());
}
