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

/// Message types for the IPC monitor
#[derive(Debug, bincode::Encode, bincode::Decode, PartialEq)]
pub enum MonMessage {
    MonEnter,
    MonLeave,
    MonWait(usize),
    MonSignal(usize),
    MonBroadcast(usize),
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct Message {
    pub sender: u32,
    pub msg: MonMessage,
}

pub const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();
pub const SIZEOF_U32: usize = std::mem::size_of::<u32>();
pub const MESSAGE_SIZE: usize = SIZEOF_U32 + 2 * SIZEOF_USIZE;

impl Message {
    pub fn new(sender: u32, msg: MonMessage) -> Self {
        Message { sender, msg }
    }

    pub fn encode(msg: Message) -> (Vec<u8>, usize) {
        let ser = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
        let len = ser.len();
        (ser, len)
    }

    pub fn decode(buffer: &[u8]) -> Result<Message, bincode::error::DecodeError> {
        let deser = bincode::decode_from_slice(buffer, bincode::config::standard());
        deser.map(|(msg, _)| msg)
    }
}
