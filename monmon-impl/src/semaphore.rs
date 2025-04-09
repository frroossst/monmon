use std::collections::LinkedList;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::{self, Thread};


#[derive(Debug)]
pub struct SemaphoreMonitor {
    pub lock: AtomicU8,
    pub condvars: LinkedList<usize>,
    pub urgentq: LinkedList<Thread>,
    pub enterq: LinkedList<Thread>,
}

impl SemaphoreMonitor {
    pub fn new(num_conds: usize) -> Self {
        let mut condvars = LinkedList::new();
        for i in 0..num_conds {
            condvars.push_back(i);
        }
        SemaphoreMonitor {
            lock: AtomicU8::new(0),
            condvars,
            urgentq: LinkedList::new(),
            enterq: LinkedList::new(),
        }
    }

    pub fn enter(&mut self) {
        loop {
            if self.lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                break;
            }
            self.enterq.push_back(thread::current());
            thread::park();
        }
    }

    pub fn leave(&mut self) {
        self.lock.store(0, Ordering::Release);
        if let Some(next_thread) = self.urgentq.pop_front().or_else(|| self.enterq.pop_front()) {
            next_thread.unpark();
        }
    }

    pub fn wait(&mut self, cv: usize) {
        if !self.condvars.contains(&cv) {
            panic!("Invalid condition variable");
        }
        self.urgentq.push_back(thread::current());
        self.leave();
        thread::park();
        self.enter();
    }

    pub fn signal(&mut self, cv: usize) {
        if !self.condvars.contains(&cv) {
            panic!("Invalid condition variable");
        }
        if let Some(thread) = self.urgentq.pop_front() {
            thread.unpark();
        }
    }
}
