use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::yield_now;


#[derive(Debug)]
pub struct BinarySemaphore {
    value: AtomicUsize,
}

impl BinarySemaphore {
    pub fn new(value: usize) -> Self {
        BinarySemaphore { value: AtomicUsize::new(value) }
    }

    #[allow(non_snake_case)]
    pub fn P_wait(&self) {
        // Spin until we can decrement the semaphore
        while self.value.load(Ordering::SeqCst) == 0 {
            // Yield to other threads instead of busy-waiting
            yield_now();
        }
        
        // Try to decrement the value atomically
        // Keep trying until we succeed
        let mut current = self.value.load(Ordering::SeqCst);
        while current > 0 {
            match self.value.compare_exchange(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return, // Successfully decremented
                Err(actual) => current = actual, // Try again with the new value
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn V_signal(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }
}


/*
 * ############################################################################
 * #                                                                          #
 * # Specific synchronised traits custom for each monitor                     #
 * #                                                                          #
 * ############################################################################
 */
/// this is typically where user code goes which runs _inside_ the monitor
pub trait Synchronised {
    fn increment(&mut self, condition: usize);
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
    // fn broadcast();
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
    RustStdlib,
    InterProcessCommunication,
}

pub struct SharedMonitor {
    monitor: UnsafeCell<Box<dyn Monitor + Send>>,
}

unsafe impl Sync for SharedMonitor {}

impl SharedMonitor {
    pub fn new(kind: MonitorKind, num_conds: usize) -> Self {
        let mon = match kind {
            MonitorKind::Semaphore => {
                SemaphoreMonitor::new(num_conds)
            }
            _ => unimplemented!(),
        };

        SharedMonitor {
            monitor: UnsafeCell::new(Box::new(mon)),
        }
    }

    pub fn with_monitor<F>(&self, f: F)
    where
        F: FnOnce(&mut SemaphoreMonitor),
    {
        unsafe {
            f(&mut *(self.monitor.get() as *mut SemaphoreMonitor));
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

    sem_enter: BinarySemaphore,
    sem_urgent: BinarySemaphore,
    urgent_count: usize,
    /// it is upto the user of the monitor to implement the mapping of semantic
    /// meaning to actual condition variables
    condvars: Vec<Condition>,
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
            sem_enter: BinarySemaphore::new(0),
            sem_urgent: BinarySemaphore::new(0),
            urgent_count: 0,
            condvars,
        }
    }
}

impl Monitor for SemaphoreMonitor {
    fn enter(&mut self) {
        if self.urgent_count > 0 {
            self.sem_enter.P_wait();
        }
        self.mutex.P_wait();
    }

    fn leave(&mut self) {
        if self.urgent_count > 0 {
            self.urgent_count -= 1;
            self.sem_urgent.V_signal();
        } else {
            self.mutex.V_signal();
        }
    }

    fn wait(&mut self, condition: usize) {
        let cv = self.condvars.get_mut(condition).unwrap();
        cv.waiting += 1;

        if self.urgent_count > 0 {
            self.sem_urgent.V_signal();
        } else {
            self.mutex.V_signal();
        }
        
        cv.sem.P_wait();
        cv.waiting -= 1;
    }

    fn signal(&mut self, condition: usize) {
        let cv = self.condvars.get_mut(condition).unwrap();

        if cv.waiting > 0 {
            self.urgent_count += 1;
            cv.sem.V_signal();
            self.sem_urgent.P_wait();
        }
    }
}

impl Synchronised for SemaphoreMonitor {
    fn increment(&mut self, _condition: usize) {
       unimplemented!() ;
    }
}
