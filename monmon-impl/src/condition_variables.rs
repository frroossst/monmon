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
