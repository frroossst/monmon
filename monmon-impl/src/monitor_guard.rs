use crate::monitors::Monitor;

/// RAII guard that locks a monitor on creation and unlocks it on drop
/// This ensures that the monitor is always properly released and is more
/// rusty/idiomatic than manually calling enter/leave.
pub struct MonitorGuard<'a, M: Monitor> {
    monitor: &'a M,
}

impl<'a, M: Monitor> MonitorGuard<'a, M> {
    pub fn new(monitor: &'a M) -> Self {
        monitor.enter();
        MonitorGuard { monitor }
    }
}

impl<'a, M: Monitor> Drop for MonitorGuard<'a, M> {
    fn drop(&mut self) {
        self.monitor.leave();
    }
}
