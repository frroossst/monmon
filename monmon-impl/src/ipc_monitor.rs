#![allow(dead_code)]
/// IPC based monitor implementation
///
///
///
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

use crate::{
    condition_variables::Condition,
    message::{Message, MonMessage},
    monitors::Monitor,
};

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
    listener: UnixListener,
}

impl IPCMonitorServer {
    pub fn new(socket_path: &str, num_conds: u32) -> Result<Self, std::io::ErrorKind> {
        let mut condvars: Vec<Condition> = Vec::with_capacity(
            num_conds
                .try_into()
                .expect("num_conds u32 must be convertible to usize"),
        );

        for _cv in 0..num_conds {
            let condition = Condition::default();
            condvars.push(condition);
        }

        let listener = match UnixListener::bind(socket_path) {
            Ok(listener) => listener,
            Err(e) => return Err(e.kind()),
        };

        Ok(Self {
            state: MonitorState {
                mutex_holder: None,
                enter_queue: Vec::new(),
                condvars,
                clients: HashMap::new(),
                next_client_id: 0..,
            },
            listener,
        })
    }

    pub fn receive(&mut self) -> Message {
        // Accept a new connection and read from it
        let (mut stream, _) = self.listener.accept().expect("Failed to accept connection");

        let mut buf = Vec::new();
        stream.read_exact(&mut buf).expect("Failed to read message");
        
        // Store the client connection for potential future use
        let client_id = self.state.next_client_id.next().unwrap();
        self.state.clients.insert(client_id, stream);
        
        Message::decode(&buf).expect("Failed to decode message")
    }
}

/// Client side of IPC monitor implementation
pub struct IPCMonitorClient {
    conn: RefCell<UnixStream>,
    id: Option<u32>,
}

impl IPCMonitorClient {
    pub fn new(conn: UnixStream) -> Self {
        IPCMonitorClient {
            conn: RefCell::new(conn),
            id: None,
        }
    }

    pub fn register(&mut self) {
    }

    pub fn send(&self, msg: MonMessage) {
        if self.id.is_none() {
            panic!("Client not registered with server, must call register() first");
        }

        let ser = Message::new(msg);
        let bytes = Message::encode(ser);

        self.conn
            .borrow_mut()
            .write_all(&bytes)
            .expect("Failed to send message");

    }
}

impl Monitor for IPCMonitorClient {
    fn enter(&self) {
        self.send(MonMessage::MonEnter);
    }

    fn leave(&self) {
        self.send(MonMessage::MonLeave);
    }

    fn wait(&self, cv: usize) {
        // compile time assertion ensures all 
        self.send(MonMessage::MonWait(cv));
    }

    fn signal(&self, cv: usize) {
        self.send(MonMessage::MonSignal(cv));
    }

    fn broadcast(&self, _cv: usize) {
        unimplemented!()
    }
}
