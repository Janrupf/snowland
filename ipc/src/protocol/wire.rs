use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum ClientMessage {}

#[derive(Debug, Encode, Decode)]
pub enum ServerMessage {
    Heartbeat,
}

pub trait IPCMessage: std::fmt::Debug + Decode + Encode + Sized {}

impl IPCMessage for ClientMessage {}
impl IPCMessage for ServerMessage {}
