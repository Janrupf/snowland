use crate::protocol::structure::Structure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    QueryConfiguration,
    QueryDisplays,
    ReorderModules(usize, usize),
    ChangeConfiguration(ChangeConfiguration),
    AddModule(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateConfiguration(Configuration),
    UpdateDisplays(Vec<Display>),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub primary: bool,
}

pub trait IPCMessage: std::fmt::Debug + Serialize + Sized {}

impl IPCMessage for ClientMessage {}
impl IPCMessage for ServerMessage {}
