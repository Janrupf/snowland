use serde::{Deserialize, Serialize};
use crate::protocol::structure::Structure;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    QueryConfiguration,
    ReorderModules(usize, usize),
    ChangeConfiguration(ChangeConfiguration),
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
    pub configuration: Structure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeConfiguration {
    pub module: usize,
    pub new_configuration: Structure,
}

pub trait IPCMessage: std::fmt::Debug + Serialize + Sized {}

impl IPCMessage for ClientMessage {}
impl IPCMessage for ServerMessage {}
