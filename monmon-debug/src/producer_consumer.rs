use std::{cell::UnsafeCell, sync::Arc, thread};

use colored::Colorize;

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

fn unsafe_multi_threaded_buffer(config: Arc<Config>) -> Box<RaceCondition<i64>> {
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


