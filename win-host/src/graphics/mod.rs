use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetDCEx, ReleaseDC, DCX_CACHE, DCX_LOCKWINDOWUPDATE, DCX_WINDOW, HDC, PFD_DOUBLEBUFFER,
    PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA,
};
use windows::Win32::Graphics::OpenGL::{ChoosePixelFormat, SetPixelFormat, PIXELFORMATDESCRIPTOR};

pub use skia_wgl::*;
pub use wgl::*;

use crate::WinApiError;

mod skia_wgl;
mod wgl;

/// Represents a graphics context centered around a [`HDC`].
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

    /// Creates a WGL context on this graphics device context.
    pub fn create_wgl_context(&self) -> Result<WGLContext, Error> {
        let descriptor = PIXELFORMATDESCRIPTOR {
            nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA as u8,
            cColorBits: 24,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 32,
            cStencilBits: 0,
            cAuxBuffers: 0,
            iLayerType: PFD_MAIN_PLANE as u8,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0,
        };

        let pixel_format = unsafe { ChoosePixelFormat(self.handle, &descriptor) };

        if pixel_format == 0 {
            return Err(Error::NoPixelFormat(WinApiError::last()));
        }

        if !unsafe { SetPixelFormat(self.handle, pixel_format, &descriptor) }.as_bool() {
            return Err(Error::PixelFormatNotChanged(WinApiError::last()));
        }

        Ok(WGLContext::for_dc(self.handle)?)
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

    #[error("failed to choose a pixel format: {0}")]
    NoPixelFormat(WinApiError),

    #[error("failed to set the pixel format: {0}")]
    PixelFormatNotChanged(WinApiError),

    #[error("failed to create WGL context: {0}")]
    WGLCreationFailed(#[from] wgl::Error),
}
