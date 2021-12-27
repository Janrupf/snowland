use crate::graphics::{XLibError, XScreen};
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
}

impl Drop for XDisplay {
    fn drop(&mut self) {
        log::trace!("Dropping XLibDisplay {:p}", self.handle);
        unsafe { xlib_sys::XCloseDisplay(self.handle) };
    }
}
