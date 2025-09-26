#[cfg(test)]
pub mod tests {

    use crate::message::{Message, MonMessage::{self, MonEnter}, MESSAGE_SIZE};

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
    }
}
