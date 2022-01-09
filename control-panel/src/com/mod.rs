//! UI communication module
mod dart_to_native;
mod ipc_state_event_channel;

pub use dart_to_native::*;
pub use ipc_state_event_channel::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error(transparent)]
    NativeShell(#[from] nativeshell::Error),
}
