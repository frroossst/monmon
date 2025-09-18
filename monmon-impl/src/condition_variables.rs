use std::sync::atomic::{AtomicU32, Ordering};
use atomic_wait::{wait, wake_one, wake_all};


use crate::semaphore::BinarySemaphore;


#[derive(Debug)]
pub struct Condition {
    pub waiting: usize,
    pub sem: BinarySemaphore,
}

impl Default for Condition {
    fn default() -> Self {
        Condition {
            waiting: 0,
            sem: BinarySemaphore::new(0),
        }
    }
}


/// A condition variable implementation using futexes
#[derive(Debug)]
pub struct FutexCondition {
    /// Number of threads waiting on this condition
    waiting: AtomicU32,
    /// Futex word for this condition variable
    futex_word: AtomicU32,
}

impl Default for FutexCondition {
    fn default() -> Self {
        Self {
            waiting: AtomicU32::new(0),
            futex_word: AtomicU32::new(0),
        }
    }
}

impl FutexCondition {
    /// Wait on this condition variable
    pub fn wait(&self) {
        // Increment waiting count
        self.waiting.fetch_add(1, Ordering::AcqRel);
        
        // Get current futex value
        let current_val = self.futex_word.load(Ordering::Acquire);
        
        // Wait on the futex
        wait(&self.futex_word, current_val);
        
        // We've been woken up, decrement waiting count
        self.waiting.fetch_sub(1, Ordering::AcqRel);
    }

    /// Signal one waiting thread
    pub fn signal(&self) -> bool {
        let waiting_count = self.waiting.load(Ordering::Acquire);
        if waiting_count > 0 {
            // Change the futex word value to wake waiters
            self.futex_word.fetch_add(1, Ordering::AcqRel);
            // Wake one waiter
            wake_one(&self.futex_word);
            true
        } else {
            false
        }
    }

    /// Broadcast to all waiting threads
    pub fn broadcast(&self) -> u32 {
        let waiting_count = self.waiting.load(Ordering::Acquire);
        if waiting_count > 0 {
            // Change the futex word value to wake waiters
            self.futex_word.fetch_add(1, Ordering::AcqRel);
            // Wake all waiters
            wake_all(&self.futex_word);
            waiting_count
        } else {
            0
        }
    }

    pub fn waiting_count(&self) -> u32 {
        self.waiting.load(Ordering::Acquire)
    }
}
