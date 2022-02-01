use std::fmt::{Debug, Formatter, Pointer};
use std::marker::PhantomData;

use bincode::serde::Compat;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, BOOL, ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND,
    ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_NOT_FOUND, ERROR_NO_DATA, ERROR_PIPE_BUSY,
    ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE, WIN32_ERROR,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, ReadFile, WriteFile, FH_OVERLAPPED, FILE_FLAG_FIRST_PIPE_INSTANCE,
    FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING,
    PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeA, DisconnectNamedPipe, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
};
use windows::Win32::System::IO::{CancelIo, GetOverlappedResult, OVERLAPPED};

use crate::platform::buffer::IPCBuffer;
use crate::{ClientMessage, IPCMessage, ServerMessage, SnowlandIPCError};

mod buffer;

const PIPE_NAME: &str = "\\\\.\\pipe\\snowland";
const BINCODE_CONFIGURATION: bincode::config::Configuration =
    bincode::config::Configuration::standard();

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
        let handle = unsafe {
            CreateNamedPipeA(
                PIPE_NAME,
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                // TODO: Maybe this can actually be a message pipe?
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                8192,
                8192,
                0,
                std::ptr::null(),
            )
        };

        // Check for creation failures, either snowland is running already or another error occurred
        if handle == INVALID_HANDLE_VALUE {
            return Err(Self::translate_open_error(
                /* to duplicated if */ ERROR_ACCESS_DENIED,
                None,
            ));
        }

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
        let handle = unsafe {
            CreateFileA(
                PIPE_NAME,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_NONE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                None,
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(Self::translate_open_error(
                /* to duplicated if */ ERROR_PIPE_BUSY,
                /* to disconnected if */ Some(ERROR_FILE_NOT_FOUND),
            ));
        }

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
                    self.buffer.mark_transferred(transferred as _);
                    self.current_read_overlapped = None;

                    self.buffer
                        .decode_available_messages(BINCODE_CONFIGURATION)
                        .map_err(|decode_err| {
                            log::error!(
                                "Disconnecting because corrupted messages were received: {}",
                                decode_err
                            );
                            self.do_disconnect();

                            decode_err.into()
                        })
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

        assert!(!result, "Asynchronous ReadFile should never return TRUE!");

        match unsafe { GetLastError() } {
            // Transfer started!
            ERROR_IO_PENDING => Ok(Vec::new()),

            // Something went wrong...
            err => {
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
        let encoded = bincode::encode_to_vec(Compat(message), BINCODE_CONFIGURATION)?;

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

    fn translate_open_error(
        duplicated_error: WIN32_ERROR,
        disconnected_error: Option<WIN32_ERROR>,
    ) -> SnowlandIPCError {
        let last_error = unsafe { GetLastError() };

        match last_error {
            e if e == duplicated_error => SnowlandIPCError::Duplicated,
            e if Some(e) == disconnected_error => SnowlandIPCError::Disconnected,
            e => SnowlandIPCError::Io(std::io::Error::from_raw_os_error(e.0 as _)),
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
