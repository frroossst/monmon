#[cfg(test)]
pub mod tests {
    use std::os::unix::net::UnixStream;

    use crate::message::MonMessage::MonEnter;
    use crate::message::{Message, MonMessage};
    use crate::monitors::{IPCMonitorClient, IPCMonitorServer};

    #[test]
    fn simple_encode_decode_test() {
        let msg = Message::new(0, MonEnter);
        let (encode, _sz) = Message::encode(msg);
        let decoded = Message::decode(&encode).expect("Failed to decode message");

        assert_eq!(decoded.msg, MonEnter);
    }

    #[test]
    fn simple_domain_socket_read_write() {
        let stream = "/tmp/ipc_monitor_test.sock";

        // Clean up any existing socket file (ignore errors)
        let _ = std::fs::remove_file(stream);

        let mut server = IPCMonitorServer::new(stream, 0).unwrap();
        let client = IPCMonitorClient::new(UnixStream::connect(stream).unwrap());

        client.send(MonMessage::MonEnter);

        let msg = server.receive();
        assert_eq!(msg.msg, MonEnter);

        // Clean up the socket file after test
        let _ = std::fs::remove_file(stream);
    }
}
