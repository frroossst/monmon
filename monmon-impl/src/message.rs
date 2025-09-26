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

#[derive(Debug)]
pub enum EncDecError {
    CorruptedMessage,
    InvalidMessage,
}

#[derive(Debug, PartialEq)]
/// Message types for the IPC monitor
pub enum MonMessage {
    MonEnter,
    MonLeave,
    MonWait(usize),
    MonSignal(usize),
}

#[derive(Debug)]
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
        let sender_bytes = msg.sender.to_be_bytes();

        let mut cv_bytes = (0_usize).to_be_bytes();
        let msg_bytes = match msg.msg {
            MonMessage::MonEnter => (0_usize).to_be_bytes(),
            MonMessage::MonLeave => (1_usize).to_be_bytes(),
            MonMessage::MonWait(cv) => {
                cv_bytes = cv.to_be_bytes();
                (2_usize).to_be_bytes()
            }
            MonMessage::MonSignal(cv) => {
                cv_bytes = cv.to_be_bytes();
                (3_usize).to_be_bytes()
            }
        };

        let mut buffer = [0u8; SIZEOF_U32 + 2 * SIZEOF_USIZE];

        buffer[0..SIZEOF_U32].copy_from_slice(&sender_bytes);
        buffer[SIZEOF_U32..SIZEOF_U32 + SIZEOF_USIZE].copy_from_slice(&msg_bytes);
        buffer[SIZEOF_U32 + SIZEOF_USIZE..SIZEOF_U32 + 2 * SIZEOF_USIZE].copy_from_slice(&cv_bytes);

        (buffer.to_vec(), MESSAGE_SIZE)
    }

    pub fn decode(buffer: &[u8]) -> Result<Message, EncDecError> {
        let sender_bytes: [u8; SIZEOF_U32] = buffer[0..std::mem::size_of::<u32>()]
            .try_into()
            .map_err(|_| EncDecError::CorruptedMessage)?;

        let msg_bytes: [u8; SIZEOF_USIZE] = buffer
            [std::mem::size_of::<u32>()..std::mem::size_of::<u32>() + std::mem::size_of::<usize>()]
            .try_into()
            .map_err(|_| EncDecError::CorruptedMessage)?;

        let cv_bytes: [u8; SIZEOF_USIZE] = buffer[std::mem::size_of::<u32>()
            + std::mem::size_of::<usize>()
            ..std::mem::size_of::<u32>() + 2 * std::mem::size_of::<usize>()]
            .try_into()
            .map_err(|_| EncDecError::CorruptedMessage)?;

        let sender = u32::from_be_bytes(sender_bytes);
        let msg_type = usize::from_be_bytes(msg_bytes);
        let cv = usize::from_be_bytes(cv_bytes);

        let msg = match msg_type {
            0 => MonMessage::MonEnter,
            1 => MonMessage::MonLeave,
            2 => MonMessage::MonWait(cv),
            3 => MonMessage::MonSignal(cv),
            _ => return Err(EncDecError::InvalidMessage),
        };

        Ok(Message { sender, msg })
    }
}
