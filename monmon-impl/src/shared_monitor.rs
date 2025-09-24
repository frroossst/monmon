use std::cell::UnsafeCell;

use crate::monitor_trait::{Monitor, MonitorKind};
use crate::semaphore_monitor::SemaphoreMonitor;
use crate::futex_monitor::FutexMonitor;

/// A thread-safe wrapper for monitor implementations that allows 
/// sharing monitors across threads safely
pub struct SharedMonitor {
    monitor: UnsafeCell<Box<dyn Monitor + Send>>,
}

unsafe impl Sync for SharedMonitor {}

impl SharedMonitor {
    /// Create a new SharedMonitor with the specified implementation type
    pub fn new(kind: MonitorKind, num_conds: usize) -> Self {
        let mon: Box<dyn Monitor + Send> = match kind {
            MonitorKind::Semaphore => Box::new(SemaphoreMonitor::new(num_conds)),
            MonitorKind::Futex => Box::new(FutexMonitor::new(num_conds)),
            _ => unimplemented!(),
        };
        SharedMonitor {
            monitor: UnsafeCell::new(mon),
        }
    }

    /// Enter the monitor (acquire exclusive access)
    pub fn enter(&self) {
        unsafe {
            (&*self.monitor.get()).enter();
        }
    }

    /// Leave the monitor (release exclusive access)
    pub fn leave(&self) {
        unsafe {
            (&*self.monitor.get()).leave();
        }
    }

    /// Wait on a specific condition variable
    pub fn wait(&self, condition: usize) {
        unsafe {
            (&*self.monitor.get()).wait(condition);
        }
    }

    /// Signal a specific condition variable
    pub fn signal(&self, condition: usize) {
        unsafe {
            (&*self.monitor.get()).signal(condition);
        }
    }
}