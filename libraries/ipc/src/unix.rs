//! Unix specific IPC backend.

use crate::SnowlandIPCError;
use std::io::Read;
use std::marker::PhantomData;

#[cfg(not(feature = "poll"))]
use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(feature = "poll")]
use mio::net::{UnixListener, UnixStream};

#[cfg(feature = "poll")]
use std::io::Write;
use std::num::ParseIntError;

use crate::protocol::{ClientMessage, IPCMessage, ServerMessage};
use bincode::error::DecodeError;
use bincode::serde::Compat;
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
        let instances = Self::list_alive_instances();
        let instance = (1..).find(|i| !instances.contains(i)).unwrap();

        let socket_path = Self::determine_socket_path(instance);

        if !socket_path.parent().map(|p| p.exists()).unwrap_or(true) {
            std::fs::create_dir_all(socket_path.parent().unwrap())?;
        }

        if socket_path.exists() {
            // TODO: Due to our instance id check this should never happen anymore

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
    pub fn connect_client(instance: usize) -> Result<Self, SnowlandIPCError> {
        let socket_path = Self::determine_socket_path(instance);
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
    fn determine_socket_path(instance: usize) -> PathBuf {
        let mut socket_dir = Self::determine_socket_dir();
        socket_dir.push(format!("host-ipc-{}.socket", instance));

        log::debug!("Determined socket path to be {}", socket_dir.display());

        socket_dir
    }

    fn determine_socket_dir() -> PathBuf {
        let temp_dir = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMP"))
            .unwrap_or_else(|_| String::from("/tmp"));

        let mut socket_dir = PathBuf::from(temp_dir);
        socket_dir.push("snowland");

        socket_dir
    }

    pub fn list_alive_instances() -> Vec<usize> {
        let socket_dir = Self::determine_socket_dir();
        if !socket_dir.exists() {
            return Vec::new();
        }

        let files = match socket_dir.read_dir() {
            Ok(v) => v,
            Err(err) => {
                log::warn!(
                    "Failed to check directory {} for alive hosts, assuming none: {}",
                    socket_dir.display(),
                    err
                );
                return Vec::new();
            }
        };

        let mut instances = Vec::new();

        for file_result in files {
            let file = match file_result {
                Ok(v) => v,
                Err(err) => {
                    log::warn!("Skipping failed entry in {}: {}", socket_dir.display(), err);
                    continue;
                }
            };

            if let Some(name) = file.path().file_name() {
                if let Some(name) = name.to_str() {
                    if name.starts_with("host-ipc-") && name.ends_with(".socket") {
                        log::trace!("Found candidate socket file {}", name);
                        let instance_str = &name[10..name.len() - 7];

                        let instance = match instance_str.parse::<usize>() {
                            Ok(v) => v,
                            Err(err) => {
                                log::warn!("Failed to parse instance id from {}: {}", name, err);
                                continue;
                            }
                        };

                        log::debug!("Found instance {}", instance);
                        instances.push(instance);
                    }
                }
            }
        }

        instances
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
        event: &mio::event::Event,
        registry: &mio::Registry,
    ) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
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

        match bincode::encode_into_std_write(Compat(message), stream, BINCODE_CONFIGURATION) {
            Ok(_) => {
                stream.flush()?;
                Ok(())
            }
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
                stream.set_nonblocking(true)?;
                self.stream = Some(stream);
                Ok(true)
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
            Err(err) => Err(SnowlandIPCError::Io(err)),
        }
    }

    #[cfg(not(feature = "poll"))]
    pub fn nonblocking_read(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
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

        match bincode::encode_into_std_write(Compat(message), stream, BINCODE_CONFIGURATION) {
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

    fn decode_messages(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let mut decoded = Vec::new();

        while self.read_buffer_real_size > 0 {
            let (message, read_size) = match bincode::decode_from_slice(
                &self.read_buffer[0..self.read_buffer_real_size],
                BINCODE_CONFIGURATION,
            ) {
                Ok((Compat(v), s)) => (v, s),
                Err(DecodeError::UnexpectedEnd) => {
                    log::trace!("Failed to decode more messages as not enough data is available");
                    break;
                }
                Err(err) => return Err(SnowlandIPCError::DecodeFailed(err)),
            };

            decoded.push(message);

            self.read_buffer_real_size -= read_size;
            self.read_buffer.drain(0..read_size);
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
