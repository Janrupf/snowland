use std::marker::PhantomData;

use crate::{ClientMessage, IPCMessage, ServerMessage, SnowlandIPCError};

#[derive(Debug)]
pub struct SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    _x: PhantomData<(S, R)>,
}

impl SnowlandIPCBackend<ServerMessage, ClientMessage> {
    pub fn create_server() -> Result<Self, SnowlandIPCError> {
        // TODO
        Ok(Self { _x: PhantomData })
    }
}

impl SnowlandIPCBackend<ClientMessage, ServerMessage> {
    pub fn connect_client() -> Result<Self, SnowlandIPCError> {
        // TODO
        Ok(Self { _x: PhantomData })
    }
}

impl<S, R> SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    #[cfg(feature = "poll")]
    pub fn register(&mut self, registry: &mio::Registry) -> Result<(), SnowlandIPCError> {
        // TODO
        Ok(())
    }

    #[cfg(feature = "poll")]
    pub fn consumes_event(&self, event: &mio::event::Event) -> bool {
        // TODO
        false
    }

    #[cfg(feature = "poll")]
    pub fn process_event(
        &mut self,
        event: &mio::event::Event,
        registry: &mio::Registry,
    ) -> Result<Vec<R>, SnowlandIPCError> {
        // TODO:
        Ok(Vec::new())
    }

    #[cfg(feature = "poll")]
    pub fn evented_write(
        &mut self,
        message: S,
        registry: &mio::Registry,
    ) -> Result<(), SnowlandIPCError> {
        // TODO
        Ok(())
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_accept(&mut self) -> Result<bool, SnowlandIPCError> {
        // TODO
        Ok(false)
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_read(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        // TODO
        Ok(Vec::new())
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_write(&mut self, message: S) -> Result<(), SnowlandIPCError> {
        // TODO
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        // TODO
        true
    }
}
