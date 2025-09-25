#![allow(dead_code)]
/// IPC based monitor implementation
///
///
///
use once_cell::sync::OnceCell;
use std::sync::atomic::AtomicBool;


static IPCSERVER_SINGLETON: OnceCell<AtomicBool> = OnceCell::new();

/// Implementing the monitor abstraction using IPC
/// Uses Send/Receive/Reply, Send(s) are blocking
#[derive(Debug)]
pub struct IPCMonitorServer {
}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {
}

