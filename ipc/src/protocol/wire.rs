use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    QueryConfiguration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateConfiguration(Configuration),
    Heartbeat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub modules: Vec<InstalledModule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledModule {
    pub ty: String,
}

pub trait IPCMessage: std::fmt::Debug + Serialize + Sized {}

impl IPCMessage for ClientMessage {}
impl IPCMessage for ServerMessage {}
