#[cfg(test)]
pub mod tests {
    use crate::message::Message;
    use crate::message::MonMessage::{MonEnter, MonLeave, MonWait, MonSignal};


    #[test]
    fn simple_encode_decode_test() {
        let msg = Message::new(0, MonEnter);

        let (encode, bytes) = Message::encode(msg);
        assert_eq!(encode, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes, 24);

        let decoded_msg = Message::decode(&encode).unwrap();
        assert_eq!(decoded_msg.sender, 0);
        assert_eq!(decoded_msg.msg, MonEnter);



        let msg = Message::new(1, MonLeave);

        let (encode, bytes) = Message::encode(msg);
        assert_eq!(encode, vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes, 24);

        let decoded_msg = Message::decode(&encode).unwrap();
        assert_eq!(decoded_msg.sender, 1);
        assert_eq!(decoded_msg.msg, MonLeave);
    }

    #[test]
    fn cv_encode_decode_test() {
        let msg = Message::new(6, MonWait(9));

        let (encode, bytes) = Message::encode(msg);
        assert_eq!(encode, vec![0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 9]);
        assert_eq!(bytes, 24);

        let decoded_msg = Message::decode(&encode).unwrap();
        assert_eq!(decoded_msg.sender, 6);
        assert_eq!(decoded_msg.msg, MonWait(9));


        let msg = Message::new(86, MonSignal(10));

        let (encode, bytes) = Message::encode(msg);
        assert_eq!(encode, vec![0, 0, 0, 0, 0, 0, 0, 86, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 10]);
        assert_eq!(bytes, 24);

        let decoded_msg = Message::decode(&encode).unwrap();
        assert_eq!(decoded_msg.sender, 86);
        assert_eq!(decoded_msg.msg, MonSignal(10));
    }

}