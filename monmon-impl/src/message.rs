use std::num::NonZero;

/*
 * ############################################################################
 * #                                                                          #
 * # Communication implementation trait                                       #
 * #                                                                          #
 * ############################################################################
 */
/// Common methods used by the communication abstraction
pub trait Communication {
    /// Blocking send operation
    fn receive(&self) -> MonMessage;
    fn send(&self, msg: MonMessage, dest: u32);
    fn reply(&self, msg: MonMessage);
}

pub const MESSAGE_SIZE: usize = 16;

/// Message types for the IPC monitor
#[derive(Debug, bincode::Encode, bincode::Decode, PartialEq)]
pub enum MonMessage {
    MonRegister,
    MonEnter,
    MonLeave,
    MonWait(usize),
    MonSignal(usize),
    MonBroadcast(usize),
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct Message {
    pub sender: NonZero<u64>,
    pub msg: MonMessage,
}

impl Message {
    /// The common use case does not need to specify the sender
    /// it automatically uses the current thread ID
    /// But an API is provided to specify it manually if needed
    /// Underllying type is simple a `NonZero<u64>` to be compatible with ThreadId::as_u64()
    pub fn new(msg: MonMessage) -> Self {
        let tid = std::thread::current().id();
        Message { 
            sender: tid.as_u64(),
            msg }
    }

    pub fn set_sender(&mut self, sender: NonZero<u64>) {
        self.sender = sender;
    }

    pub fn encode(msg: Message) -> Vec<u8> {
        let mut ser = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
        assert!(ser.len() <= MESSAGE_SIZE, "Message too large to encode");
        ser.resize(MESSAGE_SIZE, 0);
        ser
    }

    pub fn decode(buffer: &[u8]) -> Result<Message, bincode::error::DecodeError> {
        let deser = bincode::decode_from_slice(buffer, bincode::config::standard());
        deser.map(|(msg, _)| msg)
    }
}
