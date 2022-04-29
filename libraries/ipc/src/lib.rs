//! Snowland desktop IPC.

#[cfg(feature = "poll")]
pub use mio as snowland_mio;

#[cfg(feature = "poll")]
pub use mio_misc as snowland_mio_misc;

#[path = "unix.rs"]
#[cfg(unix)]
mod platform;

#[path = "windows/nonblocking.rs"]
#[cfg(all(windows, not(feature = "poll")))]
mod platform;

#[path = "windows/poll.rs"]
#[cfg(all(windows, feature = "poll"))]
mod platform;

use platform::SnowlandIPCBackend;

pub mod protocol;

#[cfg(feature = "poll")]
pub use mio;

use crate::protocol::{ClientMessage, IPCMessage, ServerMessage};
use thiserror::Error;

#[derive(Debug)]
pub struct SnowlandIPC<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    backend: SnowlandIPCBackend<S, R>,
}

impl SnowlandIPC<ServerMessage, ClientMessage> {
    /// Attempts to create the snowland IPC server.
    pub fn create_server() -> Result<Self, SnowlandIPCError> {
        Ok(Self {
            backend: SnowlandIPCBackend::create_server()?,
        })
    }
}

impl SnowlandIPC<ClientMessage, ServerMessage> {
    /// Attempts to connect the IPC client to the server.
    pub fn connect_client() -> Result<Self, SnowlandIPCError> {
        Ok(Self {
            backend: SnowlandIPCBackend::connect_client()?,
        })
    }
}

impl<S, R> SnowlandIPC<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    /// Registers the IPC instance with the given registry for receiving read/write
    /// events.
    #[cfg(feature = "poll")]
    pub fn register(&mut self, registry: &mio::Registry) -> Result<(), SnowlandIPCError> {
        self.backend.register(registry)
    }

    /// Determines whether the event is consumed by this ipc.
    #[cfg(feature = "poll")]
    pub fn consumes_event(&self, event: &mio::event::Event) -> bool {
        self.backend.consumes_event(event)
    }

    /// Processes an event and attempts to extract messages from it.
    #[cfg(feature = "poll")]
    pub fn process_event(
        &mut self,
        event: &mio::event::Event,
        registry: &mio::Registry,
    ) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        self.backend.process_event(event, registry)
    }

    /// Writes to the IPC end.
    #[cfg(feature = "poll")]
    pub fn evented_write(
        &mut self,
        message: S,
        registry: &mio::Registry,
    ) -> Result<(), SnowlandIPCError> {
        self.backend.evented_write(message, registry)
    }

    /// Attempts to accept an IPC client.
    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_accept(&mut self) -> Result<bool, SnowlandIPCError> {
        self.backend.nonblocking_accept()
    }

    /// Attempts to read incoming messages in a nonblocking way.
    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_read(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        self.backend.nonblocking_read()
    }

    /// Writes a message to the client.
    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_write(&mut self, message: S) -> Result<(), SnowlandIPCError> {
        self.backend.nonblocking_write(message)
    }

    /// Determines whether the IPC is currently connected.
    pub fn is_connected(&self) -> bool {
        self.backend.is_connected()
    }
}

#[derive(Debug, Error)]
pub enum SnowlandIPCError {
    #[error("the IPC server is started already or a client is already connected")]
    Duplicated,

    #[error("the IPC remote end is disconnected")]
    Disconnected,

    #[error("received an invalid packet of length {0}")]
    InvalidPacketLength(u32),

    #[error("expected to read {0} bytes from packet, but read {1} bytes")]
    PacketLengthMismatch(u32, usize),

    #[error("an I/O error occurred on the IPC: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to encode packet: {0}")]
    EncodeFailed(#[from] bincode::error::EncodeError),

    #[error("failed to decode packet: {0}")]
    DecodeFailed(#[from] bincode::error::DecodeError),
}
