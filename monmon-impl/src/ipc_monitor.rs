#![allow(dead_code)]
/// IPC based monitor implementation
///
/// Implements the monitor abstraction using blocking Send/Receive/Reply
/// over Unix domain sockets. A server thread maintains all monitor state
/// and controls which clients are blocked or unblocked, providing
/// Hoare-style semantics for signal and Mesa-style for broadcast.
use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::message::{Message, MonMessage, MESSAGE_SIZE};
use crate::monitor_trait::Monitor;

static SERVER_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Single ACK byte sent as reply to unblock a client
const ACK: [u8; 1] = [0xAC];

/// Commands sent from reader/acceptor threads to the server loop
enum ServerCommand {
    /// New client connected — deliver its write-end stream to the server
    Register {
        client_id: u32,
        write_stream: UnixStream,
    },
    /// A message from a registered client
    ClientMsg { client_id: u32, msg: MonMessage },
    /// Client disconnected
    Disconnect { client_id: u32 },
}

/// Internal monitor state managed exclusively by the server thread
struct MonitorState {
    /// Which client currently holds the monitor (None = free)
    mutex_holder: Option<u32>,
    /// Clients waiting to acquire the monitor
    enter_queue: VecDeque<u32>,
    /// Signalers waiting to resume after Hoare hand-off
    next_queue: VecDeque<u32>,
    /// Per-condition-variable waiting queues
    condvar_queues: Vec<VecDeque<u32>>,
    /// Write-end streams for replying to each client
    reply_streams: HashMap<u32, UnixStream>,
}

impl MonitorState {
    fn new(num_conditions: usize) -> Self {
        MonitorState {
            mutex_holder: None,
            enter_queue: VecDeque::new(),
            next_queue: VecDeque::new(),
            condvar_queues: (0..num_conditions).map(|_| VecDeque::new()).collect(),
            reply_streams: HashMap::new(),
        }
    }

    /// Send an ACK byte to the given client, unblocking its blocking read
    fn reply_to(&mut self, client_id: u32) {
        if let Some(stream) = self.reply_streams.get_mut(&client_id) {
            let _ = stream.write_all(&ACK);
            let _ = stream.flush();
        }
    }

    /// Release the monitor to the highest-priority waiter:
    /// next_queue first (Hoare semantics), then enter_queue
    fn hand_off(&mut self) {
        if let Some(id) = self.next_queue.pop_front() {
            self.mutex_holder = Some(id);
            self.reply_to(id);
        } else if let Some(id) = self.enter_queue.pop_front() {
            self.mutex_holder = Some(id);
            self.reply_to(id);
        } else {
            self.mutex_holder = None;
        }
    }

    fn process(&mut self, client_id: u32, msg: MonMessage) {
        match msg {
            MonMessage::MonRegister => {
                self.reply_to(client_id);
            }
            MonMessage::MonEnter => {
                if self.mutex_holder.is_none() {
                    self.mutex_holder = Some(client_id);
                    self.reply_to(client_id);
                } else {
                    // Queue the client — it stays blocked until replied to
                    self.enter_queue.push_back(client_id);
                }
            }
            MonMessage::MonLeave => {
                // Hand off the monitor, then ACK the leaver
                self.hand_off();
                self.reply_to(client_id);
            }
            MonMessage::MonWait(cond) => {
                // Add to condition queue (stays blocked), then release monitor
                self.condvar_queues[cond].push_back(client_id);
                self.hand_off();
            }
            MonMessage::MonSignal(cond) => {
                if let Some(waiter_id) = self.condvar_queues[cond].pop_front() {
                    // Hoare: signaler yields to waiter
                    self.next_queue.push_back(client_id);
                    self.mutex_holder = Some(waiter_id);
                    self.reply_to(waiter_id);
                    // Signaler stays blocked on next_queue until waiter leaves/waits
                } else {
                    // No waiters — signaler keeps the lock
                    self.reply_to(client_id);
                }
            }
            MonMessage::MonBroadcast(cond) => {
                // Mesa: move all waiters to enter_queue, broadcaster keeps lock
                while let Some(waiter_id) = self.condvar_queues[cond].pop_front() {
                    self.enter_queue.push_back(waiter_id);
                }
                self.reply_to(client_id);
            }
        }
    }
}

/// IPC Monitor Server
///
/// Spawns background threads that implement the monitor logic via
/// message passing over Unix domain sockets.
pub struct IPCMonitorServer {
    socket_path: PathBuf,
    server_handle: Option<JoinHandle<()>>,
    acceptor_handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl IPCMonitorServer {
    pub fn new(num_conditions: usize) -> Self {
        let id = SERVER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let socket_path =
            std::env::temp_dir().join(format!("monmon-ipc-{}-{}", std::process::id(), id));

        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind IPC socket");
        let shutdown = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<ServerCommand>();

        // Server processing thread — owns all monitor state
        let shutdown_server = shutdown.clone();
        let server_handle = thread::spawn(move || {
            let mut state = MonitorState::new(num_conditions);
            while let Ok(cmd) = rx.recv() {
                if shutdown_server.load(Ordering::Relaxed) {
                    break;
                }
                match cmd {
                    ServerCommand::Register {
                        client_id,
                        write_stream,
                    } => {
                        state.reply_streams.insert(client_id, write_stream);
                    }
                    ServerCommand::ClientMsg { client_id, msg } => {
                        state.process(client_id, msg);
                    }
                    ServerCommand::Disconnect { client_id } => {
                        state.reply_streams.remove(&client_id);
                    }
                }
            }
        });

        // Acceptor thread — accepts connections and spawns per-client reader threads
        let shutdown_acceptor = shutdown.clone();
        let acceptor_handle = thread::spawn(move || {
            let mut next_id = 0u32;
            for stream_result in listener.incoming() {
                if shutdown_acceptor.load(Ordering::Relaxed) {
                    break;
                }
                let stream = match stream_result {
                    Ok(s) => s,
                    Err(_) => break,
                };

                let client_id = next_id;
                next_id += 1;

                // Split: reader thread gets a clone, server gets the original for writing
                let read_stream = stream.try_clone().expect("Failed to clone stream");
                let write_stream = stream;

                let _ = tx.send(ServerCommand::Register {
                    client_id,
                    write_stream,
                });

                // Per-client reader thread
                let tx_reader = tx.clone();
                let shutdown_reader = shutdown_acceptor.clone();
                thread::spawn(move || {
                    let mut reader = read_stream;
                    loop {
                        if shutdown_reader.load(Ordering::Relaxed) {
                            break;
                        }
                        let mut buf = [0u8; MESSAGE_SIZE];
                        match reader.read_exact(&mut buf) {
                            Ok(()) => {
                                if let Ok(decoded) = Message::decode(&buf) {
                                    let _ = tx_reader.send(ServerCommand::ClientMsg {
                                        client_id,
                                        msg: decoded.msg,
                                    });
                                }
                            }
                            Err(_) => {
                                let _ =
                                    tx_reader.send(ServerCommand::Disconnect { client_id });
                                break;
                            }
                        }
                    }
                });
            }
        });

        IPCMonitorServer {
            socket_path,
            server_handle: Some(server_handle),
            acceptor_handle: Some(acceptor_handle),
            shutdown,
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for IPCMonitorServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Unblock the acceptor thread's blocking accept() call
        let _ = UnixStream::connect(&self.socket_path);
        if let Some(h) = self.acceptor_handle.take() {
            let _ = h.join();
        }
        if let Some(h) = self.server_handle.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// IPC Monitor Client
///
/// Implements the `Monitor` trait by sending messages to an `IPCMonitorServer`
/// and blocking until the server replies. Each thread lazily creates its own
/// Unix domain socket connection.
pub struct IPCMonitorClient {
    socket_path: PathBuf,
    streams: Mutex<HashMap<u64, Arc<Mutex<UnixStream>>>>,
}

impl IPCMonitorClient {
    pub fn new(socket_path: &Path) -> Self {
        IPCMonitorClient {
            socket_path: socket_path.to_path_buf(),
            streams: Mutex::new(HashMap::new()),
        }
    }

    /// Get or lazily create the per-thread connection to the server
    fn get_stream(&self) -> Arc<Mutex<UnixStream>> {
        let tid = std::thread::current().id().as_u64().get();

        // Fast path: stream already exists
        {
            let map = self.streams.lock().unwrap();
            if let Some(s) = map.get(&tid) {
                return s.clone();
            }
        }

        // Slow path: connect and register (without holding the map lock)
        let mut stream =
            UnixStream::connect(&self.socket_path).expect("Failed to connect to IPC server");
        let msg = Message::new(MonMessage::MonRegister);
        stream.write_all(&Message::encode(msg)).unwrap();
        let mut ack = [0u8; 1];
        stream.read_exact(&mut ack).unwrap();

        let arc = Arc::new(Mutex::new(stream));
        self.streams.lock().unwrap().insert(tid, arc.clone());
        arc
    }

    /// Send a monitor message and block until the server ACKs
    fn send_and_wait(&self, mon_msg: MonMessage) {
        let stream_arc = self.get_stream();
        let mut stream = stream_arc.lock().unwrap();
        let msg = Message::new(mon_msg);
        stream.write_all(&Message::encode(msg)).unwrap();
        let mut ack = [0u8; 1];
        stream.read_exact(&mut ack).unwrap();
    }
}

impl Monitor for IPCMonitorClient {
    fn enter(&self) {
        self.send_and_wait(MonMessage::MonEnter);
    }

    fn leave(&self) {
        self.send_and_wait(MonMessage::MonLeave);
    }

    fn wait(&self, condition: usize) {
        self.send_and_wait(MonMessage::MonWait(condition));
    }

    fn signal(&self, condition: usize) {
        self.send_and_wait(MonMessage::MonSignal(condition));
    }

    fn broadcast(&self, condition: usize) {
        self.send_and_wait(MonMessage::MonBroadcast(condition));
    }
}

/// Create a matched server + client pair
pub fn create_ipc_monitor(num_conditions: usize) -> (IPCMonitorServer, IPCMonitorClient) {
    let server = IPCMonitorServer::new(num_conditions);
    let client = IPCMonitorClient::new(server.socket_path());
    (server, client)
}
