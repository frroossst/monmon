use colored::Colorize;
use std::sync::Arc;

use monmon_dbg::accumulators::*;
use monmon_dbg::config::{Config, ConfigKind, RaceCondition, RaceKind};
use monmon_dbg::producer_consumer::*;

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
            let r = std::hint::black_box(stdblib_mutex_multi_threaded_accumulator(config));
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
        RaceKind::FutexMonitorAccum => {
            let r = std::hint::black_box(futex_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::FutexMonitorBuffer => {
            let r = std::hint::black_box(futex_multi_threaded_buffer(config));
            buffer_race_condition = Some(*r);
        }
        RaceKind::SyncProcMacroAccum => {
            let r = std::hint::black_box(proc_macro_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::IPCMonitorAccum => {
            let r = std::hint::black_box(ipc_monitor_multi_threaded_accumulator(config));
            accum_race_condition = Some(*r);
        }
        RaceKind::IPCMonitorBuffer => {
            let r = std::hint::black_box(ipc_monitor_multi_threaded_buffer(config));
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

    println!("{}", "=".repeat(80));
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

    #[cfg(not(miri))]
    {
        race(RaceKind::UnsafeAccum, config.clone());
        race(RaceKind::UnsafeBuffer, config.clone());
    }
    race(RaceKind::SemaphoreMonitorAccum, config.clone());
    race(RaceKind::SemaphoreMonitorBuffer, config.clone());
    race(RaceKind::FutexMonitorAccum, config.clone());
    race(RaceKind::FutexMonitorBuffer, config.clone());
    race(RaceKind::SyncProcMacroAccum, config.clone());
    race(RaceKind::StdlibMutexAccum, config.clone());
    race(RaceKind::StdlibMutexBuffer, config.clone());
    race(RaceKind::BinarySemaphoreAccum, config.clone());
    race(RaceKind::BinarySemaphoreBuffer, config.clone());
    race(RaceKind::IPCMonitorAccum, config.clone());
    race(RaceKind::IPCMonitorBuffer, config.clone());
}
