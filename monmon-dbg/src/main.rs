use colored::Colorize;
use std::sync::Arc;

use monmon_dbg::accumulators::{
    binary_semaphore_multi_threaded_accumulator, futex_multi_threaded_accumulator,
    ipc_monitor_multi_threaded_accumulator, proc_macro_multi_threaded_accumulator,
    sem_monitor_multi_threaded_accumulator, stdblib_mutex_multi_threaded_accumulator,
    unsafe_multi_threaded_accumulator,
};
use monmon_dbg::config::{Config, ConfigKind, RaceCondition, RaceKind};
use monmon_dbg::producer_consumer::{
    binary_semaphore_multi_threaded_buffer, futex_multi_threaded_buffer,
    ipc_monitor_multi_threaded_buffer, sem_monitor_multi_threaded_buffer,
    stdlib_mutex_multi_threaded_buffer, unsafe_multi_threaded_buffer,
};

fn race(racekind: &RaceKind, config: &Arc<Config>) {
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
    }

    if let Some(r) = accum_race_condition {
        print!("{r:?}");
    } else if let Some(r) = buffer_race_condition {
        print!("{r:?}");
    } else {
        unreachable!("both races are None!!!");
    }

    let elapsed = start.elapsed().as_millis();
    println!("{}", format!("{elapsed} ms").yellow());

    println!("{}", "=".repeat(80));
    println!();
}

fn main() {
    let mut args = std::env::args();
    let _program = args.next().expect("program name expected");

    let mode = args.next().unwrap_or_else(|| "fast".into());
    let config = match mode.as_str() {
        "slow" => Config::new(&ConfigKind::Slow),
        "medium" => Config::new(&ConfigKind::Medium),
        _ => Config::new(&ConfigKind::Fast),
    };

    println!("{config:?}");
    let config = Arc::new(config);

    #[cfg(not(miri))]
    {
        race(&RaceKind::UnsafeAccum, &config);
        race(&RaceKind::UnsafeBuffer, &config);
    }
    race(&RaceKind::SemaphoreMonitorAccum, &config);
    race(&RaceKind::SemaphoreMonitorBuffer, &config);
    race(&RaceKind::FutexMonitorAccum, &config);
    race(&RaceKind::FutexMonitorBuffer, &config);
    race(&RaceKind::SyncProcMacroAccum, &config);
    race(&RaceKind::StdlibMutexAccum, &config);
    race(&RaceKind::StdlibMutexBuffer, &config);
    race(&RaceKind::BinarySemaphoreAccum, &config);
    race(&RaceKind::BinarySemaphoreBuffer, &config);
    race(&RaceKind::IPCMonitorAccum, &config);
    race(&RaceKind::IPCMonitorBuffer, &config);
}
