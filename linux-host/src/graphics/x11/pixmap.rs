use crate::graphics::{XDisplay, XDrawable};

use x11::xlib as xlib_sys;

#[derive(Debug)]
pub struct XPixmap<'a> {
    handle: xlib_sys::Pixmap,
    display: &'a XDisplay,
}

impl<'a> XPixmap<'a> {
    pub unsafe fn new(handle: xlib_sys::Pixmap, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }
}

impl<'a> Drop for XPixmap<'a> {
    fn drop(&mut self) {
        unsafe { xlib_sys::XFreePixmap(self.display.handle(), self.handle) };
    }
}

impl<'a> XDrawable<'a> for XPixmap<'a> {
    fn drawable_handle(&self) -> xlib_sys::Drawable {
        self.handle
    }

    fn display(&self) -> &'a XDisplay {
        self.display
    }
}
