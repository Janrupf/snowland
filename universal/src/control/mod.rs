pub mod message_pipe;

/// Control messages send to and from snowland.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ControlMessage {
    ChangeScene(String),
    Exit
}
