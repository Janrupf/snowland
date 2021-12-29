use crate::xlib_sys;
use crate::{XDisplay, XDrawable};

#[derive(Debug)]
pub struct XGC<'a, T>
where
    T: XDrawable<'a>,
{
    handle: xlib_sys::GC,
    _drawable: &'a T,
    display: &'a XDisplay,
}

impl<'a, T> XGC<'a, T>
where
    T: XDrawable<'a>,
{
    pub unsafe fn new(handle: xlib_sys::GC, drawable: &'a T, display: &'a XDisplay) -> Self {
        Self {
            handle,
            _drawable: drawable,
            display,
        }
    }

    pub fn set_foreground(&self, foreground: u64) {
        unsafe { xlib_sys::XSetForeground(self.display.handle(), self.handle, foreground) };
    }

    pub fn set_background(&self, background: u64) {
        unsafe { xlib_sys::XSetBackground(self.display.handle(), self.handle, background) };
    }

    pub fn fill_rect(&self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            xlib_sys::XFillRectangle(
                self.display.handle(),
                self._drawable.drawable_handle(),
                self.handle,
                x,
                y,
                width,
                height,
            );
        }
    }

    pub fn handle(&self) -> xlib_sys::GC {
        self.handle
    }
}

impl<'a, T> Drop for XGC<'a, T>
where
    T: XDrawable<'a>,
{
    fn drop(&mut self) {
        unsafe { xlib_sys::XFreeGC(self.display.handle(), self.handle) };
    }
}
