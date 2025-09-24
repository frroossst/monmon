/// Common methods used by the monitor abstraction
pub trait Monitor {
    /// {
    ///     let _guard = monitor.lock();
    ///     /* drop gets called at the end of the scope and monitor.leave() is called too */
    /// }
    ///
    fn lock(&self) -> crate::monitor_guard::MonitorGuard<'_, Self>
    where
        Self: Sized,
    {
        crate::monitor_guard::MonitorGuard::new(self)
    }
    fn enter(&self);
    fn leave(&self);
    fn wait(&self, condition: usize);
    fn signal(&self, condition: usize);
    fn notify(&self, _condition: usize) {
        unimplemented!("Notify (Mesa-style signal) not implemented for this monitor type")
    }
    fn broadcast(&self, _condition: usize) {
        unimplemented!("Broadcast (Mesa-style signal all) not implemented for this monitor type")
    }
}

/// Enum to specify which type of monitor implementation to use
#[derive(Debug, Clone, Copy)]
pub enum MonitorKind {
    Semaphore,
    Futex,
    InterProcessCommunication,
}
