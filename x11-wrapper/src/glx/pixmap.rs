use crate::XDisplay;
use crate::{XDrawable, XPixmap};
use x11::glx as glx_sys;
use x11::xlib::Drawable;

#[derive(Debug)]
pub struct GLXPixmap<'a> {
    handle: glx_sys::GLXPixmap,
    backing: XPixmap<'a>,
    display: &'a XDisplay,
}

impl<'a> GLXPixmap<'a> {
    pub unsafe fn new(
        handle: glx_sys::GLXPixmap,
        backing: XPixmap<'a>,
        display: &'a XDisplay,
    ) -> Self {
        Self {
            handle,
            backing,
            display,
        }
    }

    pub fn handle(&self) -> glx_sys::GLXPixmap {
        self.handle
    }

    pub fn backing(&self) -> &XPixmap {
        &self.backing
    }
}

impl<'a> Drop for GLXPixmap<'a> {
    fn drop(&mut self) {
        unsafe { glx_sys::glXDestroyGLXPixmap(self.display.handle(), self.handle) };
    }
}

impl<'a> XDrawable<'a> for GLXPixmap<'a> {
    fn drawable_handle(&self) -> Drawable {
        self.handle
    }

    fn display(&self) -> &'a XDisplay {
        self.display
    }
}
