use std::cell::UnsafeCell;


#[derive(Debug)]
pub struct BinarySemaphore {
    value: usize,
}

impl BinarySemaphore {
    pub fn new(value: usize) -> Self {
        BinarySemaphore { value }
    }

    #[allow(non_snake_case)]
    pub fn P_wait(&mut self) {
        while self.value == 0 {}
        self.value -= 1;
    }

    #[allow(non_snake_case)]
    pub fn V_signal(&mut self) {
        self.value += 1;
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
