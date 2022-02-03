use crate::glx_sys;
use crate::XDisplay;
use crate::{XDrawable, XPixmap};
use x11::xlib::Drawable;

/// A GLX pixmap.
#[derive(Debug)]
pub struct GLXPixmap<'a> {
    handle: glx_sys::GLXPixmap,
    backing: XPixmap<'a>,
    display: &'a XDisplay,
}

impl<'a> GLXPixmap<'a> {
    /// Wraps a native GLX pixmap pointer.
    ///
    /// # Arguments
    ///
    /// * `handle` - The native GLX pixmap to wrap
    /// * `backing` - The X11 pixmap backing this GLX pixmap
    /// * `display` - The display this pixmap belongs to
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
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

    /// Retrieves the underlying native platform handle for the pixmap.
    pub fn handle(&self) -> glx_sys::GLXPixmap {
        self.handle
    }

    /// Retrieves the X11 pixmap backing this GLX pixmap.
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
