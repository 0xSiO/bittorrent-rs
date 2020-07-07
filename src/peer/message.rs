use std::mem::size_of;

pub struct Message {
    length: u32,
    id: MessageType,
    payload: Vec<u8>,
}

#[repr(u8)]
pub enum MessageType {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        let mut result =
            Vec::with_capacity(size_of::<u32>() + size_of::<MessageType>() + message.payload.len());
        result.extend(message.length.to_be_bytes().to_vec());
        result.push(message.id as u8);
        result.extend(message.payload);
        result
    }
}
