//! Unix specific IPC backend.

use crate::SnowlandIPCError;
use std::io::Read;
use std::marker::PhantomData;

#[cfg(not(feature = "poll"))]
use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(feature = "poll")]
use mio::net::{UnixListener, UnixStream};

use crate::protocol::{ClientMessage, IPCMessage, ServerMessage};
use std::path::PathBuf;

const BINCODE_CONFIGURATION: bincode::config::Configuration =
    bincode::config::Configuration::standard();

#[derive(Debug)]
pub struct SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    stream: Option<UnixStream>,
    read_buffer_real_size: usize,
    read_buffer: Vec<u8>, // TODO: A vec is probably not really efficient
    _data: PhantomData<(S, R)>,
}

#[cfg(feature = "poll")]
pub mod tokens {
    pub const ACCEPT: mio::Token = mio::Token(0x100);
    pub const CLIENT: mio::Token = mio::Token(0x101);
}

impl SnowlandIPCBackend<ServerMessage, ClientMessage> {
    /// Creates the IPC server and socket.
    pub fn create_server() -> Result<Self, SnowlandIPCError> {
        let socket_path = Self::determine_socket_path();

        if socket_path.exists() {
            /* Now this _could_ be a problem...
             * there are 2 possible scenarios:
             * 1. Snowland is running already
             * 2. The socket is a leftover from a crashed instance
             *
             * Only the first is a problem, in the second case
             * we simply delete the socket and re-crate it.
             *
             * Lets try to open the file!
             */

            match std::fs::File::open(&socket_path) {
                Ok(_) => {
                    /* Looks like snowland is still running! */
                    return Err(SnowlandIPCError::Duplicated);
                }
                Err(_) => {
                    log::warn!("Deleting old socket at {}", socket_path.display());
                    if let Err(err) = std::fs::remove_file(&socket_path) {
                        log::error!("Failed to delete old socket: {}", err);
                        log::error!("Be aware that binding to the socket may fail!");
                    }
                }
            }
        }

        let listener = UnixListener::bind(&socket_path)?;

        #[cfg(not(feature = "poll"))]
        listener.set_nonblocking(true)?;

        Ok(Self {
            socket_path,
            listener: Some(listener),
            stream: None,
            read_buffer_real_size: 0,
            read_buffer: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl SnowlandIPCBackend<ClientMessage, ServerMessage> {
    /// Connects the IPC client.
    pub fn connect_client() -> Result<Self, SnowlandIPCError> {
        let socket_path = Self::determine_socket_path();
        let stream = UnixStream::connect(&socket_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SnowlandIPCError::Disconnected
            } else {
                SnowlandIPCError::Io(e)
            }
        })?;

        #[cfg(not(feature = "poll"))]
        stream.set_nonblocking(true)?;

        Ok(Self {
            socket_path,
            listener: None,
            stream: Some(stream),
            read_buffer_real_size: 0,
            read_buffer: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl<S, R> SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    fn determine_socket_path() -> PathBuf {
        let temp_dir = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMP"))
            .unwrap_or_else(|_| String::from("/tmp"));

        let mut socket_path = PathBuf::from(temp_dir);
        socket_path.push("snowland-ipc");

        log::debug!("Determined socket path to be {}", socket_path.display());

        socket_path
    }

    #[cfg(feature = "poll")]
    pub fn register(&mut self, registry: &mio::Registry) -> Result<(), SnowlandIPCError> {
        if let Some(listener) = &mut self.listener {
            registry.register(listener, tokens::ACCEPT, mio::Interest::READABLE)?;
        }

        if let Some(stream) = &mut self.stream {
            registry.register(
                stream,
                tokens::CLIENT,
                mio::Interest::READABLE | mio::Interest::WRITABLE,
            )?;
        }

        Ok(())
    }

    #[cfg(feature = "poll")]
    pub fn consumes_event(&self, event: &mio::event::Event) -> bool {
        matches!(event.token(), tokens::ACCEPT | tokens::CLIENT)
    }

    #[cfg(feature = "poll")]
    pub fn process_event(
        &mut self,
        event: mio::event::Event,
        registry: &mio::Registry,
    ) -> Result<Vec<R>, SnowlandIPCError> {
        match event.token() {
            tokens::ACCEPT => {
                let (mut stream, _) = self
                    .listener
                    .as_ref()
                    .expect("Accept event triggered without a server")
                    .accept()?;

                registry.register(
                    &mut stream,
                    tokens::CLIENT,
                    mio::Interest::READABLE | mio::Interest::WRITABLE,
                )?;

                // TODO: Check if stream is already something!
                self.stream = Some(stream);
                Ok(Vec::new())
            }

            tokens::CLIENT => {
                let stream = self
                    .stream
                    .as_mut()
                    .expect("Read event triggered without a client");

                if event.is_read_closed() || event.is_write_closed() {
                    let _ = registry.deregister(stream);
                    return Ok(Vec::new());
                }

                if event.is_readable() {
                    self.read_to_internal_buffer()?;
                    match self.decode_messages() {
                        Ok(v) => Ok(v),
                        Err(err) => {
                            log::error!("Disconnecting due to message decode error: {}", err);
                            self.disconnect(registry);
                            Err(err)
                        }
                    }
                } else {
                    Ok(Vec::new())
                }
            }

            _ => panic!("process_event called with an event which does not belong to the IPC"),
        }
    }

    #[cfg(feature = "poll")]
    pub fn evented_write(
        &mut self,
        message: S,
        registry: &mio::Registry,
    ) -> Result<(), SnowlandIPCError> {
        let stream = match &mut self.stream {
            None => return Err(SnowlandIPCError::Disconnected),
            Some(v) => v,
        };

        match bincode::encode_into_std_write(message, stream, BINCODE_CONFIGURATION) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.disconnect(registry);
                Err(SnowlandIPCError::EncodeFailed(err))
            }
        }
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_accept(&mut self) -> Result<bool, SnowlandIPCError> {
        match self
            .listener
            .as_ref()
            .expect("nonblocking_accept called on a client IPC")
            .accept()
        {
            Ok((stream, _)) => {
                self.stream = Some(stream);
                Ok(true)
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
            Err(err) => Err(SnowlandIPCError::Io(err)),
        }
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_read(&mut self) -> Result<Vec<R>, SnowlandIPCError> {
        if self.stream.is_none() {
            return Err(SnowlandIPCError::Disconnected);
        }

        self.read_to_internal_buffer()?;

        match self.decode_messages() {
            Ok(v) => Ok(v),
            Err(err) => {
                log::error!("Disconnecting due to message decode error: {}", err);
                self.disconnect();
                Err(err)
            }
        }
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_write(&mut self, message: S) -> Result<(), SnowlandIPCError> {
        let stream = match &mut self.stream {
            None => return Err(SnowlandIPCError::Disconnected),
            Some(v) => v,
        };

        match bincode::encode_into_std_write(message, stream, BINCODE_CONFIGURATION) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.disconnect();
                Err(SnowlandIPCError::EncodeFailed(err))
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    #[cfg(feature = "poll")]
    fn disconnect(&mut self, registry: &mio::Registry) {
        if let Some(stream) = &mut self.stream {
            let _ = registry.deregister(stream);
        }

        self.stream = None;
        self.read_buffer_real_size = 0;
    }

    #[cfg(not(feature = "poll"))]
    fn disconnect(&mut self) {
        self.stream = None;
        self.read_buffer_real_size = 0;
    }

    fn read_to_internal_buffer(&mut self) -> Result<(), std::io::Error> {
        let stream = self.stream.as_mut().unwrap();

        loop {
            let available = self.read_buffer.len() - self.read_buffer_real_size;

            if available < 1024 {
                self.read_buffer.resize(self.read_buffer.len() + 1024, 0);
            }

            let data_start = &mut self.read_buffer[self.read_buffer_real_size..];
            let read = match stream.read(data_start) {
                Ok(v) if v < 1 => return Ok(()),
                Ok(v) => v,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(err) => return Err(err),
            };

            self.read_buffer_real_size += read;
        }
    }

    fn decode_messages(&mut self) -> Result<Vec<R>, SnowlandIPCError> {
        let mut decoded = Vec::new();

        while self.read_buffer_real_size > 4 {
            let packet_len = u32::from_ne_bytes([
                self.read_buffer[0],
                self.read_buffer[1],
                self.read_buffer[2],
                self.read_buffer[3],
            ]);

            if packet_len < 1 {
                return Err(SnowlandIPCError::InvalidPacketLength(packet_len));
            }

            if self.read_buffer_real_size >= (packet_len + 4) as usize {
                let data = &self.read_buffer[4..(packet_len as usize)];

                let (message, read_length) =
                    bincode::decode_from_slice(data, BINCODE_CONFIGURATION)?;

                if read_length != (packet_len as usize) {
                    return Err(SnowlandIPCError::PacketLengthMismatch(
                        packet_len,
                        read_length,
                    ));
                }

                self.read_buffer_real_size -= (packet_len + 1) as usize;
                self.read_buffer.drain(0..(packet_len + 4) as usize);

                decoded.push(message);
            } else {
                break;
            }
        }

        Ok(decoded)
    }
}

impl<S, R> Drop for SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    fn drop(&mut self) {
        if self.listener.is_some() {
            log::debug!(
                "Found own socket at {}, deleting!",
                self.socket_path.display()
            );

            if let Err(err) = std::fs::remove_file(&self.socket_path) {
                log::warn!("Failed to delete own socket: {}", err);
            }
        }
    }
}
