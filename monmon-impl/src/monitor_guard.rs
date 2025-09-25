use crate::monitors::Monitor;

/// RAII guard that locks a monitor on creation and unlocks it on drop
/// This ensures that the monitor is always properly released and is more
/// rusty/idiomatic than manually calling enter/leave.
/// ```rust
/// use std::sync::Arc;
/// use monmon_impl::semaphore_monitor::SemaphoreMonitor;
/// use monmon_impl::monitor_guard::MonitorGuard;
/// let monitor = Arc::new(SemaphoreMonitor::new(0));
/// {
///     let _guard = MonitorGuard::new(&*monitor);
/// } // drop is called here, releasing the monitor
/// ```
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
