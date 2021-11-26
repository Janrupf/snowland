use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};

/// Creates a new message pipe.
/// 
/// A message pipe is a wrapper around a 2 way unbounded channel.
pub fn message_pipe<T>() -> (MessagePipeEnd<T>, MessagePipeEnd<T>) {
    let (sender_one, receiver_one) = channel();
    let (sender_two, receiver_two) = channel();

    (
        MessagePipeEnd::new(sender_one, receiver_two),
        MessagePipeEnd::new(sender_two, receiver_one),
    )
}

/// Wrapper around a 2 way unbounded channel.
#[derive(Debug)]
pub struct MessagePipeEnd<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> MessagePipeEnd<T> {
    /// Creates a new message pipe end.
    fn new(sender: Sender<T>, receiver: Receiver<T>) -> Self {
        Self { sender, receiver }
    }

    /// Sends a message through the pipe.
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.sender.send(message)
    }

    /// Receives a message from the pipe.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}
