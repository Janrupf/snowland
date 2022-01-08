//! UI communication module
mod dart_to_native;
pub use dart_to_native::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error(transparent)]
    NativeShell(#[from] nativeshell::Error),
}
