use crate::WinApiError;
use std::fmt::{Debug, Formatter};
use thiserror::Error;
use windows::Win32::Foundation::{BOOL, HINSTANCE};
use windows::Win32::Graphics::Gdi::{HDC, WGL_SWAP_MAIN_PLANE};
use windows::Win32::Graphics::OpenGL::{
    wglCreateContext, wglDeleteContext, wglGetCurrentContext, wglGetProcAddress, wglMakeCurrent,
    wglSwapLayerBuffers, HGLRC,
};
use windows::Win32::System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryA};

struct WGLSwapIntervalEXT(unsafe extern "system" fn(interval: i32) -> BOOL);

impl Debug for WGLSwapIntervalEXT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:p}", self.0 as *const ())
    }
}

#[derive(Debug)]
pub struct WGLContext {
    dc: HDC,
    gl: HGLRC,
    opengl32: HINSTANCE,
    swap_interval_ext: Option<WGLSwapIntervalEXT>,
}

impl WGLContext {
    /// Creates a new WGL context for a given DC.
    pub fn for_dc(dc: HDC) -> Result<Self, Error> {
        let opengl32 = unsafe { LoadLibraryA("OpenGL32") };

        if opengl32.0 == 0 {
            return Err(Error::WinApi(WinApiError::last()));
        }

        let gl = unsafe { wglCreateContext(dc) };

        if gl.0 == 0 {
            unsafe { FreeLibrary(opengl32) };
            return Err(Error::WinApi(WinApiError::last()));
        }

        let mut instance = Self {
            dc,
            gl,
            opengl32,
            swap_interval_ext: None,
        };

        if let Ok(()) = instance.make_current() {
            let swap_interval_ext = instance.lookup_wgl_proc("wglSwapIntervalEXT");
            if !swap_interval_ext.is_null() {
                instance
                    .swap_interval_ext
                    .replace(unsafe { std::mem::transmute(swap_interval_ext) });
            }
        }

        Ok(instance)
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
            Err(Error::WinApi(WinApiError::last()))
        }
    }

    /// Presents the surface by swapping the front and back buffer.
    pub fn swap_buffers(&self) -> Result<(), Error> {
        let result = unsafe { wglSwapLayerBuffers(self.dc, WGL_SWAP_MAIN_PLANE) }.as_bool();

        if result {
            Ok(())
        } else {
            Err(Error::WinApi(WinApiError::last()))
        }
    }

    /// Determines whether the current context is this context.
    pub fn is_current(&self) -> bool {
        self.gl == unsafe { wglGetCurrentContext() }
    }

    /// Attempts to change the swap interval of the context.
    pub fn change_swap_interval(&self, interval: i32) -> Result<(), Error> {
        if let Some(wgl_swap_interval_ext) = &self.swap_interval_ext {
            let result = unsafe { wgl_swap_interval_ext.0(interval) }.as_bool();

            if result {
                Ok(())
            } else {
                Err(Error::WinApi(WinApiError::last()))
            }
        } else {
            Err(Error::WGLSwapControlExtNotAvailable)
        }
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

    #[error("WGL_EXT_swap_control not available")]
    WGLSwapControlExtNotAvailable,
}
