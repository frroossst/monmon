use colored::Colorize;
use std::sync::Arc;

use monmon_dbg::accumulators::*;
use monmon_dbg::producer_consumer::*;
use monmon_dbg::config::{Config, ConfigKind, RaceCondition, RaceKind};



fn race(racekind: RaceKind, config: Arc<Config>) {
    let start = std::time::Instant::now();

    let mut accum_race_condition: Option<RaceCondition<usize>> = None;
    let mut buffer_race_condition: Option<RaceCondition<i64>> = None;
    
    match racekind {
        RaceKind::UnsafeAccum => {
            let r = std::hint::black_box(unsafe_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::UnsafeBuffer => {
            let r = std::hint::black_box(unsafe_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::SemaphoreMonitorAccum => {
            let r = std::hint::black_box(sem_monitor_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::SemaphoreMonitorBuffer => {
            let r = std::hint::black_box(sem_monitor_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::StdlibMutexAccum => {
            let r =std::hint::black_box(stdblib_mutex_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::StdlibMutexBuffer => {
            let r = std::hint::black_box(stdlib_mutex_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::BinarySemaphoreAccum => {
            let r = std::hint::black_box(binary_semaphore_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::BinarySemaphoreBuffer => {
            let r = std::hint::black_box(binary_semaphore_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::HappyLockAccum => {
            let r = std::hint::black_box(happylock_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::HappyLockBuffer => {
            let r = std::hint::black_box(happylock_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::FutexAccum => {
            let r = std::hint::black_box(futex_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::FutexBuffer => {
            let r = std::hint::black_box(futex_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
    };

    if let Some(r) = accum_race_condition {
        print!("{:?}", r);
    } else if let Some(r) = buffer_race_condition {
        print!("{:?}", r);
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

    // race(RaceKind::UnsafeAccum, config.clone());
    // race(RaceKind::UnsafeBuffer, config.clone());
    race(RaceKind::SemaphoreMonitorAccum, config.clone());
    // race(RaceKind::SemaphoreMonitorBuffer, config.clone());
    // race(RaceKind::StdlibMutexAccum, config.clone());
    // race(RaceKind::StdlibMutexBuffer, config.clone());
    // race(RaceKind::BinarySemaphoreAccum, config.clone());
    // race(RaceKind::BinarySemaphoreBuffer, config.clone());
    // race(RaceKind::HappyLockAccum, config.clone());
    // race(RaceKind::HappyLockBuffer, config.clone());
    // race(RaceKind::FutexAccum, config.clone());
    // race(RaceKind::FutexBuffer, config.clone());
}
