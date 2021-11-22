use std::fmt::{Display, Formatter};
use thiserror::Error;
use windows::Win32::Foundation::{GetLastError, PSTR, WIN32_ERROR};
use windows::Win32::System::Diagnostics::Debug::{
    FormatMessageA, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};
use windows::Win32::System::Memory::LocalFree;

/// Represents a Win32 error code wrapped in a rust error.
#[derive(Debug, Error)]
pub struct WinApiError {
    code: WIN32_ERROR,
}

impl WinApiError {
    /// Creates a new rust error from an existing Win32 error code.
    pub fn new(code: WIN32_ERROR) -> Self {
        Self { code }
    }

    /// Creates a rust error for the last windows error.
    pub fn last() -> Self {
        let last_error = unsafe { GetLastError() };
        Self::new(last_error)
    }

    /// Formats the error code as a human readable message.
    pub fn format_as_message(&self) -> Result<String, WinApiError> {
        let mut message_buffer = PSTR(std::ptr::null_mut());
        let message_size = unsafe {
            FormatMessageA(
                FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS,
                std::ptr::null(),
                self.code.0,
                0,
                std::mem::transmute(&mut message_buffer),
                0,
                std::ptr::null(),
            )
        };

        if message_size == 0 {
            return Err(Self::last());
        }

        let message_buffer_slice =
            unsafe { std::slice::from_raw_parts(message_buffer.0, message_size as usize) };
        let message = String::from_utf8_lossy(message_buffer_slice);

        unsafe { LocalFree(message_buffer.0 as isize) };

        Ok(message.into())
    }
    
    /// Retrieves the underlying error code
    pub fn code(&self) -> WIN32_ERROR {
        self.code
    }
    
    /// Tests whether this error is of a certain type
    pub fn is(&self, other: WIN32_ERROR) -> bool {
        self.code == other
    }
}

impl Display for WinApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message = self.format_as_message().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", message)
    }
}
