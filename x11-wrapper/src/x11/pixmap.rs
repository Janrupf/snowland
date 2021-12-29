use crate::{XDisplay, XDrawable};

use crate::xlib_sys;

/// X11 pixmap drawable.
///
/// An X11 pixmap is a offscreen buffer which can be drawn into and copied from.
#[derive(Debug)]
pub struct XPixmap<'a> {
    handle: xlib_sys::Pixmap,
    display: &'a XDisplay,
}

impl<'a> XPixmap<'a> {
    /// Wraps an existing X11 pixmap.
    ///
    /// # Arguments
    ///
    /// * `handle` - The X11 pixmap to wrap
    /// * `display` - The display the pixmap belongs to
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
    pub unsafe fn new(handle: xlib_sys::Pixmap, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Retrieves the underlying native X11 pixmap id.
    pub fn handle(&self) -> xlib_sys::Pixmap {
        self.handle
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
