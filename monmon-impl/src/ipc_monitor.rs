#![allow(dead_code)]
/// IPC based monitor implementation
///
///
///
use std::{collections::HashMap, os::unix::net::UnixStream};

use crate::condition_variables::Condition;

/// Internal state maintained by IPCMonitorServer instance
struct MonitorState {
    mutex_holder: Option<u32>,
    enter_queue: Vec<u32>,
    condvars: Vec<Condition>,
    clients: HashMap<u32, UnixStream>,
    next_client_id: std::ops::RangeFrom<u32>,
}

/// Implementing the monitor abstraction using IPC
/// Uses Send/Receive/Reply, Send(s) are blocking
pub struct IPCMonitorServer {}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {}
