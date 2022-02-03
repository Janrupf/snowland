//! UI communication module
mod dart_to_native;
mod display_event_channel;
mod ipc_state_event_channel;
mod responder;

pub use dart_to_native::*;
pub use display_event_channel::*;
pub use ipc_state_event_channel::*;
pub use responder::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error(transparent)]
    NativeShell(#[from] nativeshell::Error),
}
