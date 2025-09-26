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
    MonRegister,
    MonEnter,
    MonLeave,
    MonWait(usize),
    MonSignal(usize),
    MonBroadcast(usize),
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct Message {
    pub sender: Option<u32>,
    pub msg: MonMessage,
}

pub const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();
pub const SIZEOF_U32: usize = std::mem::size_of::<u32>();
pub const MESSAGE_SIZE: usize = SIZEOF_U32 + 2 * SIZEOF_USIZE;

impl Message {
    pub fn new(sender: Option<u32>, msg: MonMessage) -> Self {
        Message { sender, msg }
    }

    pub fn encode(msg: Message) -> Vec<u8> {
        let mut encoded = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
        
        // Pad or truncate to MESSAGE_SIZE
        encoded.resize(MESSAGE_SIZE, 0);
        encoded
    }

    pub fn decode(buffer: &[u8]) -> Result<Message, bincode::error::DecodeError> {
        let deser = bincode::decode_from_slice(buffer, bincode::config::standard());
        deser.map(|(msg, _)| msg)
    }
}
