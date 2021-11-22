use crate::WinApiError;
use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetDCEx, ReleaseDC, DCX_CACHE, DCX_LOCKWINDOWUPDATE, DCX_WINDOW, HDC,
};

/// Represents a graphics context centered around a [`HDC`]
#[derive(Debug)]
pub struct Graphics {
    window: HWND,
    handle: HDC,
}

impl Graphics {
    /// Creates a graphics context from an existing window.
    pub fn from_window(window: HWND) -> Result<Graphics, Error> {
        let handle =
            unsafe { GetDCEx(window, None, DCX_WINDOW | DCX_CACHE | DCX_LOCKWINDOWUPDATE) };

        if handle.0 == 0 {
            Err(Error::DCRetrievalFailed(WinApiError::last()))
        } else {
            Ok(Graphics { window, handle })
        }
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        unsafe { ReleaseDC(self.window, self.handle) };
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get device context: {0}")]
    DCRetrievalFailed(WinApiError),
}
