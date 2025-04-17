use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/*
 * ############################################################################
 * #                                                                          #
 * # Atomic Binary Semaphore                                                  #
 * #                                                                          #
 * ############################################################################
 */
#[derive(Debug)]
/// A binary semaphore implementation using atomic operations
pub struct BinarySemaphore {
    count: AtomicUsize,
}

impl BinarySemaphore {
    /// Creates a new binary semaphore with the given initial value.
    pub const fn new(initial: usize) -> Self {
        BinarySemaphore {
            count: AtomicUsize::new(initial),
        }
    }

    #[allow(non_snake_case)]
    /// Wait operation (P operation) on the semaphore.
    pub fn P_wait(&self) {
        loop {
            let mut current = self.count.load(Ordering::Relaxed);
            while current == 0 {
                for _ in 0..10_000 {
                    // crude delay
                    let _ = self.count.load(Ordering::Relaxed);
                }
                current = self.count.load(Ordering::Relaxed);
            }
            if self
                .count
                .compare_exchange(current, current - 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    #[allow(non_snake_case)]
    /// Signal operation (V operation) on the semaphore.
    pub fn V_signal(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
}

/*
 * ############################################################################
 * #                                                                          #
 * # Monitor implementation trait                                             #
 * #                                                                          #
 * ############################################################################
 */
/// Common methods used by the monitor abstraction
pub trait Monitor {
    fn enter(&mut self);
    fn leave(&mut self);
    fn wait(&mut self, condition: usize);
    fn signal(&mut self, condition: usize);
    fn notify(&mut self, _condition: usize) {
        unimplemented!()
    }
    fn broadcast(&mut self, _condition: usize) {
        unimplemented!()
    }
}

/*
 * ############################################################################
 * #                                                                          #
 * # Shared Monitor to send monitors across threads                           #
 * #                                                                          #
 * ############################################################################
 */
pub enum MonitorKind {
    Semaphore,
    InterProcessCommunication,
}

pub struct SharedMonitor {
    monitor: UnsafeCell<Box<dyn Monitor + Send>>,
}

unsafe impl Sync for SharedMonitor {}

impl SharedMonitor {
    pub fn new(kind: MonitorKind, num_conds: usize) -> Self {
        let mon: Box<dyn Monitor + Send> = match kind {
            MonitorKind::Semaphore => Box::new(SemaphoreMonitor::new(num_conds)),
            _ => unimplemented!(),
        };
        SharedMonitor {
            monitor: UnsafeCell::new(mon),
        }
    }
    pub fn enter(&self) {
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&mut *self.monitor.get()).enter();
        }
    }
    pub fn leave(&self) {
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&mut *self.monitor.get()).leave();
        }
    }
    pub fn wait(&self, condition: usize) {
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&mut *self.monitor.get()).wait(condition);
        }
    }
    pub fn signal(&self, condition: usize) {
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&mut *self.monitor.get()).signal(condition);
        }
    }
}

/*
 * ====================================================================================================================
 */

/*
 * ############################################################################
 * #                                                                          #
 * # Monitor implementation semaphores                                        #
 * #                                                                          #
 * ############################################################################
 */

#[derive(Debug)]
pub struct Condition {
    waiting: usize,
    sem: BinarySemaphore,
}

/// Implementing the monitor abstraction using semaphores
#[derive(Debug)]
pub struct SemaphoreMonitor {
    /// only one thread is allowed to be _inside_ the monitor at any given time
    mutex: BinarySemaphore,

    enter_queue: BinarySemaphore,
    /// it is upto the user of the monitor to implement the mapping of semantic
    /// meaning to actual condition variables
    condvars: Vec<Condition>,

    /// number of threads waiting on the condition
    next_count: usize,
}

// Implementing Send and Sync for SemaphoreMonitor
unsafe impl Sync for SemaphoreMonitor {}

impl SemaphoreMonitor {
    pub fn new(num_conds: usize) -> Self {
        let mut condvars: Vec<Condition> = Vec::with_capacity(num_conds);
        for _cv in 0..num_conds {
            let condition = Condition {
                waiting: 0,
                sem: BinarySemaphore::new(0),
            };
            condvars.push(condition);
        }

        SemaphoreMonitor {
            mutex: BinarySemaphore::new(1),
            enter_queue: BinarySemaphore::new(0),
            condvars,
            next_count: 0,
        }
    }
}

impl Monitor for SemaphoreMonitor {
    fn enter(&mut self) {
        self.mutex.P_wait();
    }

    fn leave(&mut self) {
        if self.next_count > 0 {
            self.enter_queue.V_signal();
        } else {
            self.mutex.V_signal();
        }
    }

    fn wait(&mut self, condition: usize) {
        let cv = &mut self.condvars[condition];
        cv.waiting += 1;
        // Upon waiting, release the monitor.
        if self.next_count > 0 {
            self.enter_queue.V_signal();
        } else {
            self.mutex.V_signal();
        }
        // Block on the condition variable's semaphore.
        cv.sem.P_wait();
        cv.waiting -= 1;
        // When this thread resumes, it continues inside the monitor.
    }

    fn signal(&mut self, condition: usize) {
        let cv = &mut self.condvars[condition];
        if cv.waiting > 0 {
            self.next_count += 1;
            cv.sem.V_signal();
            // Wait for the signalled thread to re-enter the monitor.
            self.enter_queue.P_wait();
            self.next_count -= 1;
        }
        // If no thread is waiting, do nothing.
    }
}
