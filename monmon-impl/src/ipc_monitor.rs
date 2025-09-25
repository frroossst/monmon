#![allow(dead_code)]
/// IPC based monitor implementation
///
///
///
use once_cell::sync::OnceCell;
use std::{collections::HashMap, os::unix::net::UnixStream, sync::atomic::AtomicBool};

use crate::{condition_variables::Condition, message::{Message, MonMessage}};


static IPCSERVER_SINGLETON: OnceCell<AtomicBool> = OnceCell::new();

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
pub struct IPCMonitorServer {
    state: MonitorState,
}

impl IPCMonitorServer {
    pub fn new(num_conds: u32) -> Self {
        let mut condvars: Vec<Condition> = Vec::with_capacity(num_conds.try_into().expect("num_conds u32 must be convertible to usize"));
        for _cv in 0..num_conds {
            let condition = Condition::default();
            condvars.push(condition);
        }

        IPCMonitorServer {
            state: MonitorState {
                mutex_holder: None,
                enter_queue: Vec::new(),
                condvars,
                clients: HashMap::new(),
                next_client_id: 0..,
            },
        }
    }
}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {
    conn: UnixStream,
    id: Option<u32>,
}

impl IPCMonitorClient {
    pub fn new(conn: UnixStream) -> Self {
        IPCMonitorClient { conn, id: None }
    }

    pub fn send(&self, msg: MonMessage) {
        let ser = Message::new(self.id.expect("Client must be registered before sending messages"), msg);
    }

}

