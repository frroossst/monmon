#[cfg(test)]
pub mod tests {
    use std::os::unix::net::UnixStream;

    use crate::message::MonMessage::MonEnter;
    use crate::message::{Message, MonMessage};
    use crate::monitors::{IPCMonitorClient, IPCMonitorServer};

    #[test]
    fn simple_encode_decode_test() {
        let msg = Message::new(None, MonEnter);
        let encode = Message::encode(msg);
        let decoded = Message::decode(&encode).expect("Failed to decode message");

        assert_eq!(decoded.msg, MonEnter);
    }

    #[test]
    fn simple_domain_socket_read_write() {
        let socket_path = "/tmp/ipc_monitor_test.sock";

        // Clean up any existing socket file (ignore errors)
        let _ = std::fs::remove_file(socket_path);

        let mut server = IPCMonitorServer::new(socket_path, 0).unwrap();
        
        // Use a separate thread for the client to avoid blocking
        let handle = std::thread::spawn(move || {
            // Give the server a moment to start listening
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            let client = IPCMonitorClient::new(UnixStream::connect(socket_path).unwrap());
            client.send(MonMessage::MonEnter);
            
            // Keep the connection alive for a bit to ensure message is sent
            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        let msg = server.receive();
        assert_eq!(msg.msg, MonEnter);

        handle.join().unwrap();
    }
}
