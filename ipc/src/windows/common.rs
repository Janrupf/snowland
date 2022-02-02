use windows::Win32::Foundation::{
    GetLastError, ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, HANDLE,
    INVALID_HANDLE_VALUE, WIN32_ERROR,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    CreateNamedPipeA, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
};

use crate::SnowlandIPCError;

pub const PIPE_NAME: &str = "\\\\.\\pipe\\snowland";
pub const BINCODE_CONFIGURATION: bincode::config::Configuration =
    bincode::config::Configuration::standard();

pub struct WindowsPipeIo;

impl WindowsPipeIo {
    pub fn create_pipe() -> Result<HANDLE, SnowlandIPCError> {
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
            Err(Self::translate_open_error(
                /* to duplicated if */ ERROR_ACCESS_DENIED,
                None,
            ))
        } else {
            Ok(handle)
        }
    }

    pub fn connect_pipe() -> Result<HANDLE, SnowlandIPCError> {
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
            Err(Self::translate_open_error(
                /* to duplicated if */ ERROR_PIPE_BUSY,
                /* to disconnected if */ Some(ERROR_FILE_NOT_FOUND),
            ))
        } else {
            Ok(handle)
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
