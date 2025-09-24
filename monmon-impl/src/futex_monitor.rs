use std::sync::atomic::{AtomicU32, Ordering};

use atomic_wait::{wait, wake_one};

use crate::condition_variables::FutexCondition;
use crate::monitor_trait::Monitor;

/// Monitor implementation using futexes with Hoare semantics
#[derive(Debug)]
pub struct FutexMonitor {
    /// Mutex futex word (0 = unlocked, 1 = locked)
    mutex: AtomicU32,
    /// Condition variables implemented with futexes
    conditions: Vec<FutexCondition>,
    /// Number of threads in the next queue (signaled threads waiting to re-enter)
    next_count: AtomicU32,
    /// Next queue futex for signaled threads
    next_queue: AtomicU32,
}

impl FutexMonitor {
    pub fn new(num_conditions: usize) -> Self {
        let mut conditions = Vec::with_capacity(num_conditions);
        for _ in 0..num_conditions {
            conditions.push(FutexCondition::default());
        }

        FutexMonitor {
            mutex: AtomicU32::new(0), // 0 = unlocked
            conditions,
            next_count: AtomicU32::new(0),
            next_queue: AtomicU32::new(0),
        }
    }

    /// Acquire the monitor mutex using futex
    fn acquire_mutex(&self) {
        loop {
            // Try to acquire the mutex (compare_exchange from 0 to 1)
            match self
                .mutex
                .compare_exchange_weak(0, 1, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(_) => break, // Successfully acquired
                Err(current) => {
                    // Mutex is locked, wait for it
                    if current == 1 {
                        wait(&self.mutex, 1);
                    }
                }
            }
        }
    }

    /// Release the monitor mutex and wake waiters
    fn release_mutex(&self) {
        self.mutex.store(0, Ordering::Release);
        wake_one(&self.mutex);
    }

    /// Wait on the next queue
    fn wait_next_queue(&self) {
        let current_val = self.next_queue.load(Ordering::Acquire);
        wait(&self.next_queue, current_val);
    }

    /// Signal the next queue
    fn signal_next_queue(&self) {
        self.next_queue.fetch_add(1, Ordering::AcqRel);
        wake_one(&self.next_queue);
    }
}

// SAFETY: FutexMonitor uses atomic operations and futexes for synchronization
unsafe impl Sync for FutexMonitor {}

impl Monitor for FutexMonitor {
    fn enter(&self) {
        self.acquire_mutex();
    }

    fn leave(&self) {
        let next_count = self.next_count.load(Ordering::Acquire);
        if next_count > 0 {
            // There are signaled threads waiting, let one proceed
            self.signal_next_queue();
        } else {
            // No signaled threads, release the mutex normally
            self.release_mutex();
        }
    }

    fn wait(&self, condition: usize) {
        if condition >= self.conditions.len() {
            panic!("wait: Condition index {} out of bounds", condition);
        }

        // Release the monitor lock before waiting
        let next_count = self.next_count.load(Ordering::Acquire);
        if next_count > 0 {
            self.signal_next_queue();
        } else {
            self.release_mutex();
        }

        // Wait on the condition variable
        self.conditions[condition].wait();

        // After being signaled, we need to re-acquire the monitor
        // In Hoare semantics, the signaling thread passes control directly
        // This is handled by the signal method's implementation
    }

    fn signal(&self, condition: usize) {
        if condition >= self.conditions.len() {
            panic!("signal: Condition index {} out of bounds", condition);
        }

        // Check if any thread is waiting on this condition
        if self.conditions[condition].waiting_count() > 0 {
            // Increment next_count to indicate a thread will be in the next queue
            self.next_count.fetch_add(1, Ordering::AcqRel);

            // Signal the condition variable (wake one waiter)
            self.conditions[condition].signal();

            // The signaling thread now waits on the next queue
            // This implements Hoare semantics: immediate handoff of control
            self.wait_next_queue();

            // When we return, the signaled thread has finished and signaled us back
            self.next_count.fetch_sub(1, Ordering::AcqRel);
        }
        // If no thread is waiting, do nothing (continue with monitor lock held)
    }

    fn notify(&self, condition: usize) {
        if condition >= self.conditions.len() {
            panic!("notify: Condition index {} out of bounds", condition);
        }

        // Mesa-style signal: just wake the thread, don't yield control
        self.conditions[condition].signal();
    }

    fn broadcast(&self, condition: usize) {
        if condition >= self.conditions.len() {
            panic!("broadcast: Condition index {} out of bounds", condition);
        }

        // Wake all threads waiting on this condition
        let woken_count = self.conditions[condition].broadcast();

        // In Mesa semantics, we don't yield control immediately
        // All woken threads will compete for the monitor lock when we leave
        if woken_count > 0 {
            println!(
                "Broadcast woke {} threads on condition {}",
                woken_count, condition
            );
        }
    }
}
