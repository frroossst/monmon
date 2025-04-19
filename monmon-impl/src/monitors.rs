use std::cell::UnsafeCell;

use crate::semaphore::BinarySemaphore;

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
        unimplemented!("Notify (Mesa-style signal) not implemented for this monitor type")
    }
    fn broadcast(&mut self, _condition: usize) {
        unimplemented!("Broadcast (Mesa-style signal all) not implemented for this monitor type")
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

    /// Waits on a specific condition variable.
    /// 1. Increments the condition's waiting count.
    /// 2. Releases the monitor lock (prioritizing `enter_queue` waiters).
    /// 3. Blocks on the condition's semaphore (`cond.sem`).
    /// 4. When woken by `signal`, the thread implicitly holds the lock again
    ///    (due to the Hoare semantics enforced by `signal`'s wait on `enter_queue`).
    fn wait(&mut self, condition: usize) {
        // Ensure the condition index is valid.
        if condition >= self.condvars.len() {
            // Or return an error, depending on desired robustness
            panic!("wait: Condition index out of bounds");
        }

        let cond = &mut self.condvars[condition];

        // 1. Indicate intention to wait on this condition.
        cond.waiting += 1;

        // 2. Release the monitor lock. Decide who gets it next:
        //    If next_count > 0, there's a signaled thread waiting on enter_queue.
        //    Let it proceed first (Hoare semantics).
        //    Otherwise, release the main mutex for any new entrant.
        //    This is the same logic as in `leave`.
        if self.next_count > 0 {
            self.enter_queue.V_signal();
        } else {
            self.mutex.V_signal();
        }
        // At this point, this thread no longer holds the monitor lock.

        // 3. Block on the specific condition semaphore.
        //    This thread will wait here until another thread calls signal()
        //    for this condition.
        cond.sem.P_wait();

        // 4. Woken up by signal().
        //    In this Hoare-style implementation, the `signal` method ensures that
        //    *before* this thread returns from `cond.sem.P_wait()`, the signaling
        //    thread has blocked on `enter_queue`, effectively passing the monitor
        //    lock *directly* to this waiting thread.
        //    Therefore, upon waking here, this thread implicitly holds the lock again.
        //    We don't need another P_wait here.
        //    The `waiting` count was decremented by `signal`.
    }

    /// Signals a specific condition variable.
    /// If threads are waiting on the condition:
    /// 1. Decrements the condition's waiting count.
    /// 2. Signals the condition's semaphore (`cond.sem`), waking one waiter.
    /// 3. Increments `next_count` to indicate a thread is being prioritized.
    /// 4. Blocks the *signaling* thread on `enter_queue`, yielding the monitor lock
    ///    to the thread woken in step 2.
    /// 5. When the woken thread eventually leaves or waits again, it signals `enter_queue`,
    ///    waking this signaling thread back up.
    /// 6. Decrements `next_count`.
    fn signal(&mut self, condition: usize) {
        // Ensure the condition index is valid.
         if condition >= self.condvars.len() {
             panic!("signal: Condition index out of bounds");
         }

        // Only proceed if there is actually a thread waiting on this condition.
        // Crucially, check `waiting` *before* potentially blocking self on enter_queue.
        if self.condvars[condition].waiting > 0 {
            // A thread is waiting, so we will perform the signal-and-wait dance.

            // 3. Increment next_count: Indicates that we (the signaler) will soon block,
            //    and a woken thread will need to use the enter_queue mechanism.
            self.next_count += 1;

            // Borrow mutably to modify waiting count.
             let cond = &mut self.condvars[condition];

             // 1. Decrement waiting count *before* signaling.
             //    The woken thread is no longer technically waiting on the condition,
             //    it's about to be scheduled.
             cond.waiting -= 1;

            // 2. Signal the condition semaphore. This wakes up exactly one thread
            //    that is currently blocked in `cond.sem.P_wait()`.
            cond.sem.V_signal();

            // 4. Wait on the enter_queue. This blocks the *signaling* thread
            //    and crucially yields the monitor lock implicitly to the thread
            //    that was just woken up by `cond.sem.V_signal()`.
            self.enter_queue.P_wait();

            // 5. Woken up. This happens when the thread we signaled calls `leave`
            //    or `wait` again, which checks `next_count > 0` and signals `enter_queue`.
            //    We now re-acquire the monitor lock implicitly.

            // 6. Decrement next_count: We have now consumed the signal on enter_queue.
            self.next_count -= 1;
        }
        // If `cond.waiting` was 0, do nothing. The signaler continues execution
        // inside the monitor without interruption.
    }

}
