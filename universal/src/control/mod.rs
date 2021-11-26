pub mod message_pipe;

/// Control messages send to and from snowland.
#[derive(Debug)]
pub enum ControlMessage {
    ChangeScene(String),
}
