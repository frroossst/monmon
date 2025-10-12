use std::cell::UnsafeCell;

use monmon_impl::monitors::SemaphoreMonitor;
use monmon_impl::monitors::Monitor;
use monmon_proc::synchronised;

// write a struct that demonstrates the use of the synchronised macro
// use #[synchronised] on a function that takes an argument of type &impl Monitor
// the function should call enter() on the monitor at the start of the function
// and leave() on the monitor at the end of the function

pub struct SyncedStruct {
    monitor: SemaphoreMonitor,
    counter: UnsafeCell<usize>,
}

// SAFETY: This is safe because access to the counter is synchronized through the monitor
unsafe impl Sync for SyncedStruct {}

impl SyncedStruct {
    pub fn new() -> Self {
        Self {
            monitor: SemaphoreMonitor::new(1),
            counter: UnsafeCell::new(0),
        }
    }

    #[synchronised(self.monitor)]
    pub fn increment(&self) {
        // SAFETY: This is safe because access is synchronized through the monitor
        unsafe {
            let current_value = *self.counter.get();
            crate::work::do_something();
            *self.counter.get() = current_value + 1;
            crate::work::do_something();
        }
    }

    pub fn get_counter(&self) -> usize {
        // SAFETY: This is safe because we're only reading the value
        unsafe { *self.counter.get() }
    }
}
