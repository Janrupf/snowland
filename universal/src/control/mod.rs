pub mod message_pipe;

#[derive(Debug)]
pub enum ControlMessage {
    ChangeScene(String)
}