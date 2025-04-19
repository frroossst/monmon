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
