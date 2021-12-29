use crate::{XDisplay, XWindow};
use x11::xlib as xlib_sys;

#[derive(Debug)]
pub struct XScreen<'a> {
    handle: *mut xlib_sys::Screen,
    display: &'a XDisplay,
}

impl<'a> XScreen<'a> {
    pub unsafe fn new(handle: *mut xlib_sys::Screen, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    pub fn handle(&self) -> *mut xlib_sys::Screen {
        self.handle
    }

    pub fn number(&self) -> i32 {
        unsafe { xlib_sys::XScreenNumberOfScreen(self.handle) }
    }

    pub fn root_window(&self) -> XWindow {
        unsafe { XWindow::new((*self.handle).root, self.display) }
    }
}
