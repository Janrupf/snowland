use crate::graphics::{XDisplay, XWindow};
use x11::xlib as xlib_sys;
use x11::xlib::{XFillRectangle, XFreeGC};

#[derive(Debug)]
pub struct XGC<'a> {
    handle: xlib_sys::GC,
    window: &'a XWindow<'a>,
    display: &'a XDisplay,
}

impl<'a> XGC<'a> {
    pub unsafe fn new(handle: xlib_sys::GC, window: &'a XWindow, display: &'a XDisplay) -> Self {
        Self {
            handle,
            window,
            display,
        }
    }

    pub fn fill_rect(&self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            XFillRectangle(
                self.display.handle(),
                self.window.handle(),
                self.handle,
                x,
                y,
                width,
                height,
            );
        }
    }
}

impl<'a> Drop for XGC<'a> {
    fn drop(&mut self) {
        unsafe { XFreeGC(self.display.handle(), self.handle) };
    }
}
