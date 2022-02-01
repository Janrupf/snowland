use std::fmt::Debug;

use thiserror::Error;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::Graphics::Gdi::{HDC, WGL_SWAP_MAIN_PLANE};
use windows::Win32::Graphics::OpenGL::{
    wglCreateContext, wglDeleteContext, wglGetCurrentContext, wglGetProcAddress, wglMakeCurrent,
    wglSwapLayerBuffers, HGLRC,
};
use windows::Win32::System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryA};

use crate::util::WinApiError;

#[derive(Debug)]
pub struct WGLContext {
    dc: HDC,
    gl: HGLRC,
    opengl32: HINSTANCE,
}

impl WGLContext {
    /// Creates a new WGL context for a given DC.
    pub fn for_dc(dc: HDC) -> Result<Self, Error> {
        let opengl32 = unsafe { LoadLibraryA("OpenGL32") };

        if opengl32.0 == 0 {
            return Err(Error::WinApi(WinApiError::from_win32()));
        }

        let gl = unsafe { wglCreateContext(dc) };

        if gl.0 == 0 {
            unsafe { FreeLibrary(opengl32) };
            return Err(Error::WinApi(WinApiError::from_win32()));
        }

        Ok(Self { dc, gl, opengl32 })
    }

    /// Attempts to find a WGL function by first querying WGL and then the OpenGL32 library.
    pub fn lookup_wgl_proc(&self, name: &str) -> *const std::ffi::c_void {
        debug_assert!(self.is_current());
        if !self.is_current() {
            return std::ptr::null();
        }

        let proc =
            unsafe { wglGetProcAddress(name).or_else(|| GetProcAddress(self.opengl32, name)) };

        match proc {
            None => std::ptr::null(),
            Some(v) => v as _,
        }
    }

    /// Makes the context current for this thread.
    pub fn make_current(&self) -> Result<(), Error> {
        let result = unsafe { wglMakeCurrent(self.dc, self.gl) }.as_bool();

        if result {
            Ok(())
        } else {
            Err(Error::WinApi(WinApiError::from_win32()))
        }
    }

    /// Presents the surface by swapping the front and back buffer.
    pub fn swap_buffers(&self) -> Result<(), Error> {
        let result = unsafe { wglSwapLayerBuffers(self.dc, WGL_SWAP_MAIN_PLANE) }.as_bool();

        if result {
            Ok(())
        } else {
            Err(Error::WinApi(WinApiError::from_win32()))
        }
    }

    /// Determines whether the current context is this context.
    pub fn is_current(&self) -> bool {
        self.gl == unsafe { wglGetCurrentContext() }
    }
}

impl Drop for WGLContext {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.opengl32);
            wglDeleteContext(self.gl);
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("error while calling Win32: {0}")]
    WinApi(WinApiError),
}
