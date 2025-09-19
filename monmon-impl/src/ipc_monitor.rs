use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::sync::atomic::AtomicBool;
use once_cell::sync::OnceCell;

use crate::condition_variables::Condition;
use crate::message::{Message, MESSAGE_SIZE};



static IPCSERVER_SINGLETON: OnceCell<AtomicBool> = OnceCell::new();

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
            let condition = Condition::default();
            conditions.push(condition);
        }

        IPCMonitorServer {
            tx,
            rx,
            clients: Vec::new(),
            conditions,
        }
    }

    pub fn connection(&self) -> (RawFd, RawFd) {
        (self.tx.as_raw_fd(), self.rx.as_raw_fd())
    }

    /// loop {
    ///     let msg = self.receive();
    ///     
    /// }
    pub fn receive(&mut self) -> Message {
        let mut buffer = vec![0u8; MESSAGE_SIZE];
        let bytes_read = nix::unistd::read(self.rx.as_raw_fd(), &mut buffer).unwrap();
        if bytes_read != MESSAGE_SIZE {
            panic!("Failed to read the full message");
        }
        
        if let Ok(msg) = Message::decode(&buffer) {
            msg
        } else {
            eprintln!("{:?}", buffer);
            panic!("Failed to decode message");
        }
    }

    pub fn serve(&mut self) {
        let _singleton = IPCSERVER_SINGLETON.get_or_init(|| AtomicBool::new(false));

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


