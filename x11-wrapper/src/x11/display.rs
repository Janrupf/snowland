use crate::xlib_sys;
use crate::{XAtom, XLibError, XScreen};
use std::ffi::{CStr, CString};

/// The heart of an X11 connection.
///
/// In the context of XLib this represents a connection to the X11 server.
#[derive(Debug)]
pub struct XDisplay {
    handle: *mut xlib_sys::Display,
}

impl XDisplay {
    /// Attempts to open the connection to the X11 server using the default display.
    pub fn open() -> Result<Self, XLibError> {
        let handle = unsafe { xlib_sys::XOpenDisplay(std::ptr::null()) };
        if handle.is_null() {
            return Err(XLibError::OpenDisplayFailed);
        }

        Ok(XDisplay { handle })
    }

    /// Retrieves the underlying X11 native platform pointer.
    pub fn handle(&self) -> *mut xlib_sys::Display {
        self.handle
    }

    /// Retrieves the default screen of the X11 display.
    pub fn default_screen(&self) -> XScreen {
        unsafe { XScreen::new(xlib_sys::XDefaultScreenOfDisplay(self.handle), self) }
    }

    /// Synchronizes the X11 command queue and flushes all commands.
    ///
    /// This function will call the error handlers for any outstanding errors.
    ///
    /// # Arguments
    ///
    /// * `discard` - If `true`, outstanding commands will be discarded instead of flushed
    pub fn sync(&self, discard: bool) {
        unsafe { xlib_sys::XSync(self.handle, discard.into()) };
    }

    /// Attempts to retrieve an existing X11 atom from the display.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the atom to retrieve
    ///
    /// # Panics
    ///
    /// If the name contains a nul character.
    pub fn get_atom(&self, name: impl AsRef<str>) -> Option<XAtom> {
        let name = CString::new(name.as_ref()).unwrap();
        let atom = unsafe { xlib_sys::XInternAtom(self.handle, name.as_ptr(), 1) };

        if atom == 0 {
            None
        } else {
            Some(unsafe { XAtom::new(atom) })
        }
    }

    /// Attempts to retrieve an X11 atom from the display, creating it if it doesn't exist yet.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the atom to retrieve or create
    ///
    /// # Panics
    ///
    /// If the name contains a nul character.
    pub fn get_or_create_atom(&self, name: impl AsRef<str>) -> XAtom {
        let name = CString::new(name.as_ref()).unwrap();
        let atom = unsafe { xlib_sys::XInternAtom(self.handle, name.as_ptr(), 0) };

        debug_assert!(atom != 0);
        unsafe { XAtom::new(atom) }
    }
}

impl Drop for XDisplay {
    fn drop(&mut self) {
        unsafe { xlib_sys::XCloseDisplay(self.handle) };
    }
}
