/// IPC based monitor implementation
///
///
///
use once_cell::sync::OnceCell;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::sync::atomic::AtomicBool;

use crate::condition_variables::Condition;
use crate::message::{MESSAGE_SIZE, Message};

static IPCSERVER_SINGLETON: OnceCell<AtomicBool> = OnceCell::new();

/// Implementing the monitor abstraction using IPC
/// Uses Send/Receive/Reply, Send(s) are blocking
#[derive(Debug)]
pub struct IPCMonitorServer {
}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {
}

