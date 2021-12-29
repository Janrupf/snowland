use crate::{XAtom, XLibError, XScreen};
use std::ffi::{CStr, CString};
use x11::xlib as xlib_sys;

#[derive(Debug)]
pub struct XDisplay {
    handle: *mut xlib_sys::Display,
}

impl XDisplay {
    pub fn open() -> Result<Self, XLibError> {
        let handle = unsafe { xlib_sys::XOpenDisplay(std::ptr::null()) };
        if handle.is_null() {
            return Err(XLibError::OpenDisplayFailed);
        }

        Ok(XDisplay { handle })
    }

    pub fn handle(&self) -> *mut xlib_sys::Display {
        self.handle
    }

    pub fn default_screen(&self) -> XScreen {
        unsafe { XScreen::new(xlib_sys::XDefaultScreenOfDisplay(self.handle), self) }
    }

    pub fn sync(&self, discard: bool) {
        unsafe { xlib_sys::XSync(self.handle, discard.into()) };
    }

    pub fn get_atom(&self, name: impl AsRef<str>) -> Option<XAtom> {
        let name = CString::new(name.as_ref()).unwrap();
        let atom = unsafe { xlib_sys::XInternAtom(self.handle, name.as_ptr(), 1) };

        if atom == 0 {
            None
        } else {
            Some(unsafe { XAtom::new(atom) })
        }
    }

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
