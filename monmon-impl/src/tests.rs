#[cfg(test)]
pub mod tests {

    use crate::{
        message::{
            MESSAGE_SIZE, Message,
            MonMessage::{self, MonEnter},
        },
        monitors::{Monitor, create_ipc_monitor},
    };

    #[test]
    fn simple_encode_decode_test() {
        let msg = Message::new(MonEnter);
        let encode = Message::encode(msg);
        let decoded = Message::decode(&encode).expect("Failed to decode message");

        assert_eq!(decoded.msg, MonEnter);
    }

    #[test]
    fn message_size_test() {
        let msg = Message::new(MonEnter);
        let ser = Message::encode(msg);
        assert!(ser.len() <= MESSAGE_SIZE);

        let msg = Message::new(MonEnter);
        let ser = Message::encode(msg);
        assert!(ser.len() <= MESSAGE_SIZE);

        let msg = Message::new(MonMessage::MonWait(u32::MAX as usize));
        let ser = Message::encode(msg);
        assert!(ser.len() == MESSAGE_SIZE);
    }

    #[test]
    fn simple_domain_socket_read_write() {
        use std::io::{Read, Write};
        use std::os::unix::net::UnixStream;

        let (mut writer, mut reader) = UnixStream::pair().unwrap();

        let msg = Message::new(MonEnter);
        let encoded = Message::encode(msg);
        writer.write_all(&encoded).unwrap();

        let mut buf = [0u8; MESSAGE_SIZE];
        reader.read_exact(&mut buf).unwrap();
        let decoded = Message::decode(&buf).unwrap();
        assert_eq!(decoded.msg, MonEnter);
    }

    #[test]
    fn ipc_monitor_enter_leave() {
        let (_server, client) = create_ipc_monitor(0);
        client.enter();
        client.leave();
    }

    #[test]
    fn ipc_monitor_concurrent_enter_leave() {
        use std::sync::Arc;

        let (_server, client) = create_ipc_monitor(0);
        let client = Arc::new(client);
        let mut handles = vec![];

        for _ in 0..4 {
            let c = client.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..10 {
                    c.enter();
                    c.leave();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
