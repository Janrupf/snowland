use std::io::{Read, Write};
use std::marker::PhantomData;
use std::os::windows::io::FromRawHandle;

use bincode::serde::Compat;
use mio::windows::NamedPipe;

use crate::platform::buffer::IPCBuffer;
use crate::platform::common::WindowsPipeIo;
use crate::{ClientMessage, IPCMessage, ServerMessage, SnowlandIPCError};

mod buffer;
mod common;

#[cfg(feature = "poll")]
pub mod tokens {
    pub const ACCEPT: mio::Token = mio::Token(0x100);
    pub const CLIENT: mio::Token = mio::Token(0x101);
}

#[derive(Debug)]
pub struct SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    pipe: NamedPipe,
    server: bool,
    connected: bool,
    buffer: IPCBuffer,
    write_buffer: Vec<u8>,
    _data: PhantomData<(S, R)>,
}

impl SnowlandIPCBackend<ServerMessage, ClientMessage> {
    pub fn create_server() -> Result<Self, SnowlandIPCError> {
        // Create a named pipe for the server
        let handle = WindowsPipeIo::create_pipe()?;
        let pipe = unsafe { NamedPipe::from_raw_handle(handle.0 as _) };

        // Make the pipe ready for connections...
        let connected = if let Err(err) = pipe.connect() {
            if err.kind() != std::io::ErrorKind::WouldBlock {
                return Err(SnowlandIPCError::Io(err));
            }
            false
        } else {
            true
        };

        Ok(Self {
            pipe,
            server: true,
            connected,
            buffer: IPCBuffer::new(),
            write_buffer: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl SnowlandIPCBackend<ClientMessage, ServerMessage> {
    pub fn connect_client() -> Result<Self, SnowlandIPCError> {
        // Attempt to connect to the named pipe
        let handle = WindowsPipeIo::connect_pipe()?;
        let pipe = unsafe { NamedPipe::from_raw_handle(handle.0 as _) };

        Ok(Self {
            pipe,
            server: false,
            connected: true, // At this point we are always connected
            buffer: IPCBuffer::new(),
            write_buffer: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl<S, R> SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    pub fn register(&mut self, registry: &mio::Registry) -> Result<(), SnowlandIPCError> {
        registry.register(
            &mut self.pipe,
            tokens::CLIENT,
            mio::Interest::READABLE | mio::Interest::WRITABLE,
        );

        Ok(())
    }

    pub fn consumes_event(&self, event: &mio::event::Event) -> bool {
        matches!(event.token(), tokens::ACCEPT | tokens::CLIENT)
    }

    pub fn process_event(
        &mut self,
        event: &mio::event::Event,
        registry: &mio::Registry,
    ) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        match event.token() {
            tokens::ACCEPT => {
                panic!("tokens::ACCEPT is never used by the windows poll implementation")
            }
            tokens::CLIENT => {
                if event.is_read_closed() || event.is_write_closed() {
                    self.do_disconnect(registry);

                    return if self.server {
                        log::info!("IPC client disconnected, making pipe ready again!");

                        if let Err(err) = self.pipe.connect() {
                            if err.kind() != std::io::ErrorKind::WouldBlock {
                                log::error!(
                                    "Failed to make pipe ready for connection again: {}",
                                    err
                                );
                                return Err(SnowlandIPCError::Io(err));
                            }
                        }

                        Ok(Vec::new())
                    } else {
                        log::warn!("Disconnected as server went away!");
                        Err(SnowlandIPCError::Disconnected)
                    };
                }

                if event.is_readable() {
                    if let Err(err) = self.buffer.read_using(|buffer| self.pipe.read(buffer)) {
                        log::error!(
                            "Disconnecting due to error while reading from pipe: {}",
                            err
                        );
                        self.do_disconnect(registry);

                        return Err(SnowlandIPCError::Io(err));
                    }

                    // TODO: check for read of 0 bytes, as this seems to mean the pipe is closed

                    self.buffer
                        .decode_available_messages(common::BINCODE_CONFIGURATION)
                        .map_err(|decode_err| {
                            log::error!(
                                "Disconnecting due to receiving corrupted messages: {}",
                                decode_err
                            );
                            self.do_disconnect(registry);

                            decode_err.into()
                        })
                } else {
                    if event.is_writable() {
                        if self.server {
                            log::info!("IPC client connected!");
                            self.connected = true;

                            Ok(Vec::new())
                        } else {
                            log::trace!("Client pipe writable!");
                            match self.write_out_buffer() {
                                Ok(()) => Ok(Vec::new()),
                                Err(err) => {
                                    log::error!("Disconnecting due to write error: {}", err);
                                    self.do_disconnect(registry);

                                    Err(err)
                                }
                            }
                        }
                    } else {
                        Ok(Vec::new())
                    }
                }
            }

            _ => panic!("process_event called with an event which does not belong to the IPC"),
        }
    }

    pub fn evented_write(
        &mut self,
        message: S,
        registry: &mio::Registry,
    ) -> Result<(), SnowlandIPCError> {
        if !self.connected {
            return Err(SnowlandIPCError::Disconnected);
        }

        bincode::encode_into_std_write(
            Compat(message),
            &mut self.write_buffer,
            common::BINCODE_CONFIGURATION,
        )?;

        match self.write_out_buffer() {
            Ok(()) => Ok(()),
            Err(err) => {
                log::error!("Disconnecting due to write failure: {}", err);
                self.do_disconnect(registry);

                Err(err)
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    fn do_disconnect(&mut self, registry: &mio::Registry) {
        self.buffer.flush();

        if self.server {
            let _ = self.pipe.disconnect();
        } else {
            let _ = registry.deregister(&mut self.pipe);
        }

        self.connected = false;
    }

    fn write_out_buffer(&mut self) -> Result<(), SnowlandIPCError> {
        while !self.write_buffer.is_empty() {
            let write_size = match self.pipe.write(&self.write_buffer) {
                Ok(w) => w,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => return Err(SnowlandIPCError::Io(err)),
            };

            self.write_buffer.drain(0..write_size);
        }

        Ok(())
    }
}
