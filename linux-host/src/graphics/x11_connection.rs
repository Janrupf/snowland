use thiserror::Error;
use x11::xlib::{Display, XCloseDisplay, XDefaultScreen, XOpenDisplay, XSync};

#[derive(Debug)]
pub struct XLibDisplay {
    display: *mut Display,
}

impl XLibDisplay {
    pub fn open() -> Result<Self, XLibError> {
        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            return Err(XLibError::OpenDisplayFailed);
        }

        Ok(XLibDisplay { display })
    }

    pub fn handle(&self) -> *mut Display {
        self.display
    }

    pub fn default_screen(&self) -> i32 {
        unsafe { XDefaultScreen(self.display) }
    }

    pub fn sync(&self, discard: bool) {
        unsafe { XSync(self.display, discard.into()) };
    }
}

impl Drop for XLibDisplay {
    fn drop(&mut self) {
        log::trace!("Dropping XLibDisplay {:p}", self.display);
        unsafe { XCloseDisplay(self.display) };
    }
}

#[derive(Debug, Error)]
pub enum XLibError {
    #[error("failed to open display")]
    OpenDisplayFailed,
}
