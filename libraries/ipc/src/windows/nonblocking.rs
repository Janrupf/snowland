use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use bincode::serde::Compat;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_NO_DATA,
    ERROR_PIPE_CONNECTED, HANDLE,
};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows::Win32::System::Pipes::{ConnectNamedPipe, DisconnectNamedPipe};
use windows::Win32::System::IO::{CancelIo, GetOverlappedResult, OVERLAPPED};

use buffer::IPCBuffer;
use common::WindowsPipeIo;

use crate::{ClientMessage, IPCMessage, ServerMessage, SnowlandIPCError};

mod buffer;
mod common;

pub struct SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    pipe: HANDLE,
    server: bool,
    connected: bool,
    buffer: IPCBuffer,
    current_read_overlapped: Option<OVERLAPPED>,
    pending_overlapped_writes: Vec<(OVERLAPPED, Vec<u8>)>,
    _data: PhantomData<(S, R)>,
}

impl SnowlandIPCBackend<ServerMessage, ClientMessage> {
    pub fn create_server() -> Result<Self, SnowlandIPCError> {
        // Create a named pipe the client will later connect to
        let handle = WindowsPipeIo::create_pipe()?;

        Ok(Self {
            pipe: handle,
            server: true,
            connected: false,
            buffer: IPCBuffer::new(),
            current_read_overlapped: None,
            pending_overlapped_writes: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl SnowlandIPCBackend<ClientMessage, ServerMessage> {
    pub fn connect_client() -> Result<Self, SnowlandIPCError> {
        // Attempt to connect to the named pipe
        let handle = WindowsPipeIo::connect_pipe()?;

        Ok(Self {
            pipe: handle,
            server: false,
            connected: true, // At this point we are always connected
            buffer: IPCBuffer::new(),
            current_read_overlapped: None,
            pending_overlapped_writes: Vec::new(),
            _data: PhantomData,
        })
    }
}

impl<S, R> SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    pub fn nonblocking_accept(&mut self) -> Result<bool, SnowlandIPCError> {
        if !self.server {
            panic!("nonblocking_accept called on a client IPC");
        } else if self.connected {
            return Ok(false);
        }

        let mut overlapped = OVERLAPPED::default();
        let result = unsafe { ConnectNamedPipe(self.pipe, &mut overlapped) }.as_bool();
        match (result, unsafe { GetLastError() }) {
            // A client connected now, good!
            (true, _) => {
                self.connected = true;
                Ok(true)
            }

            // No client connected yet, also good...
            (false, ERROR_IO_PENDING) => Ok(false),

            // A client connected before ConnectNamedPipe, good again...
            (false, ERROR_PIPE_CONNECTED) => {
                self.connected = true;
                Ok(true)
            }

            (false, ERROR_NO_DATA) => {
                // Client disappeared before it really connected..
                log::warn!("An IPC client connected and disconnected immediately!");
                self.do_disconnect();
                Ok(false)
            }

            // Something went wrong, not good!
            (false, err) => Err(SnowlandIPCError::Io(std::io::Error::from_raw_os_error(
                err.0 as _,
            ))),
        }
    }

    pub fn nonblocking_read(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        if !self.connected {
            return Err(SnowlandIPCError::Disconnected);
        }

        if self.current_read_overlapped.is_some() {
            // We are already performing overlapped io...

            let mut transferred = 0;

            let completed = unsafe {
                GetOverlappedResult(
                    self.pipe,
                    self.current_read_overlapped.as_ref().unwrap(),
                    &mut transferred,
                    false,
                )
            }
            .as_bool();

            return match (completed, unsafe { GetLastError() }) {
                (true, _) => {
                    // Operation completed!
                    log::trace!("Nonblocking read {} bytes asynchronously!", transferred);
                    self.buffer.mark_transferred(transferred as _);
                    self.current_read_overlapped = None;

                    self.decode_available()
                }
                (false, ERROR_IO_PENDING) | (false, ERROR_IO_INCOMPLETE) => {
                    // We are still transferring...
                    Ok(Vec::new())
                }
                (false, err) => {
                    log::error!(
                        "Disconnecting because asynchronous write operation failed: 0x{:X}",
                        err.0
                    );
                    self.do_disconnect();

                    Err(SnowlandIPCError::Io(std::io::Error::from_raw_os_error(
                        err.0 as _,
                    )))
                }
            };
        }

        let overlapped = self.current_read_overlapped.insert(OVERLAPPED::default());

        // Ready to start new transfer!
        let transfer_buffer = self.buffer.prepare_transfer_buffer();
        let result = unsafe {
            ReadFile(
                self.pipe,
                transfer_buffer.as_mut_ptr() as _,
                transfer_buffer.len() as _,
                std::ptr::null_mut(),
                overlapped,
            )
        }
        .as_bool();

        match (result, unsafe { GetLastError() }) {
            // Transfer completed!
            (true, _) => {
                let mut read_count = 0;

                let completed =
                    unsafe { GetOverlappedResult(self.pipe, overlapped, &mut read_count, false) }
                        .as_bool();

                assert!(
                    completed,
                    "ReadFile returned true without operation being completed!"
                );

                self.current_read_overlapped = None;

                log::trace!("Nonblocking read {} bytes synchronously", read_count);
                self.buffer.mark_transferred(read_count as _);

                self.decode_available()
            }

            // Transfer started!
            (false, ERROR_IO_PENDING) => {
                log::trace!(
                    "Started async transfer of max {} bytes",
                    transfer_buffer.len()
                );

                Ok(Vec::new())
            }

            // Something went wrong...
            (false, err) => {
                log::error!(
                    "Disconnecting because asynchronous read operation failed: 0x{:X}",
                    err.0
                );
                self.do_disconnect();

                Err(SnowlandIPCError::Io(std::io::Error::from_raw_os_error(
                    err.0 as _,
                )))
            }
        }
    }

    pub fn nonblocking_write(&mut self, message: S) -> Result<(), SnowlandIPCError> {
        if !self.connected {
            return Err(SnowlandIPCError::Disconnected);
        }

        self.process_pending_writes()?;

        let mut overlapped = OVERLAPPED::default();
        let encoded = bincode::encode_to_vec(Compat(message), common::BINCODE_CONFIGURATION)?;

        let result = unsafe {
            WriteFile(
                self.pipe,
                encoded.as_ptr() as _,
                encoded.len() as _,
                std::ptr::null_mut(),
                &mut overlapped,
            )
        }
        .as_bool();

        match (result, unsafe { GetLastError() }) {
            // Transfer completed already!
            (true, _) => Ok(()),

            // Transfer started!
            (false, ERROR_IO_PENDING) => {
                self.pending_overlapped_writes.push((overlapped, encoded));

                Ok(())
            }

            // Something went wrong...
            (false, err) => {
                log::error!(
                    "Disconnecting because asynchronous write operation failed: 0x{:X}",
                    err.0
                );
                self.do_disconnect();

                Err(SnowlandIPCError::Io(std::io::Error::from_raw_os_error(
                    err.0 as _,
                )))
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    fn decode_available(&mut self) -> Result<Vec<R>, SnowlandIPCError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        self.buffer
            .decode_available_messages(common::BINCODE_CONFIGURATION)
            .map_err(|decode_err| {
                log::error!(
                    "Disconnecting because corrupted messages were received: {}",
                    decode_err
                );
                self.do_disconnect();

                decode_err.into()
            })
    }

    fn process_pending_writes(&mut self) -> Result<(), SnowlandIPCError> {
        // NOTE: Reverse iterator to speed things up... this also reverses the order errors are
        //       returned in, though this is not really important for our use case anyway.
        for i in (0..self.pending_overlapped_writes.len()).rev() {
            let result = unsafe {
                let (overlapped, _) = &self.pending_overlapped_writes[i];

                let mut transferred = 0;
                GetOverlappedResult(self.pipe, overlapped, &mut transferred, false)
            }
            .as_bool();

            match (result, unsafe { GetLastError() }) {
                // I/O is done
                (true, _) => {
                    self.pending_overlapped_writes.remove(i);
                }
                (false, ERROR_IO_PENDING) | (false, ERROR_IO_INCOMPLETE) => {}
                // Something went wrong
                (false, err) => {
                    log::error!(
                        "Disconnecting because asynchronous write operation failed: 0x{:X}",
                        err.0
                    );
                    self.do_disconnect();

                    self.pending_overlapped_writes.remove(i);
                    return Err(SnowlandIPCError::Io(std::io::Error::from_raw_os_error(
                        err.0 as _,
                    )));
                }
            }
        }

        Ok(())
    }

    fn do_disconnect(&mut self) {
        unsafe { CancelIo(self.pipe) };

        self.connected = false;
        self.buffer.flush();
        self.pending_overlapped_writes.clear();
        self.current_read_overlapped = None;

        if self.server {
            unsafe {
                DisconnectNamedPipe(self.pipe);
            }
        } else {
            unsafe { CloseHandle(self.pipe) };
        }
    }
}

impl<S, R> Debug for SnowlandIPCBackend<S, R>
where
    S: IPCMessage,
    R: IPCMessage,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        struct DebugOptionNonExhaustive<'a, T>(&'a Option<T>);

        impl<'a, T> Debug for DebugOptionNonExhaustive<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match &self.0 {
                    Some(_) => write!(f, "Some(...)"),
                    None => write!(f, "None"),
                }
            }
        }

        struct DebugVecNonExhaustive<'a, T>(&'a Vec<T>);

        impl<'a, T> Debug for DebugVecNonExhaustive<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let len = self.0.len();
                write!(f, "[{{{}}}]", len)
            }
        }

        f.debug_struct("SnowlandIPCBackend<S, R>")
            .field("pipe", &self.pipe)
            .field("connected", &self.connected)
            .field("buffer", &self.buffer)
            .field(
                "current_read_overlapped",
                &DebugOptionNonExhaustive(&self.current_read_overlapped),
            )
            .field(
                "pending_overlapped_writes",
                &DebugVecNonExhaustive(&self.pending_overlapped_writes),
            )
            .field("_data", &self._data)
            .finish()
    }
}
