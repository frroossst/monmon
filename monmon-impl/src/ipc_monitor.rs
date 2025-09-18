use std::os::fd::{AsRawFd, OwnedFd, RawFd};

use crate::condition_variables::Condition;
use crate::message::{Message, MESSAGE_SIZE};
use crate::semaphore::BinarySemaphore;

static mut IPCSERVER_SINGLETON: bool = false;

/// Implementing the monitor abstraction using IPC
/// Uses Send/Receive/Reply, Send(s) are blocking 
#[derive(Debug)]
pub struct IPCMonitorServer {
    tx: OwnedFd,
    rx: OwnedFd,
    clients: Vec<OwnedFd>,
    conditions: Vec<Condition>,
}

impl IPCMonitorServer {
    pub fn new(num_conds: usize) -> Self {
        let (tx, rx) = nix::unistd::pipe().unwrap();

        let mut conditions: Vec<Condition> = Vec::with_capacity(num_conds);
        for _ in 0..num_conds {
            let condition = Condition {
                waiting: 0,
                sem: BinarySemaphore::new(0),
            };
            conditions.push(condition);
        }

        IPCMonitorServer {
            tx,
            rx,
            clients: Vec::new(),
            conditions,
        }
    }

    pub fn receive(&mut self) -> Message {
        // Blocking receive operation
        let mut buffer = vec![0u8; MESSAGE_SIZE];
        let bytes_read = nix::unistd::read(self.rx.as_raw_fd(), &mut buffer).unwrap();
        if bytes_read != MESSAGE_SIZE {
            panic!("Failed to read the full message");
        }
        Message::decode(&buffer).unwrap()
    }

    pub fn serve(&mut self) {
        // Check if the server is already running
        if unsafe { IPCSERVER_SINGLETON } {
            panic!("IPCMonitorServer is already running");
        }
        unsafe { IPCSERVER_SINGLETON = true; }

        loop {
            let msg = self.receive();
            dbg!(msg);
        }
    }
}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {
    tx: RawFd,
    rx: RawFd,
}

impl IPCMonitorClient {
    pub fn new(tx: RawFd, rx: RawFd) -> Self {
        unimplemented!()
    }
}