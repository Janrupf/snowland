use crate::xlib_sys;
use crate::{XDisplay, XWindow};

/// X11 screen.
///
/// Please note that while originally screens where meant to represent different heads (monitors)
/// on an X system, they rarely do anymore. Usually all monitors are combined as one huge screen
/// and the window manager takes care of assigning application windows to monitors.
///
/// Thus you can usually expect one X11 display to have one screen!
#[derive(Debug)]
pub struct XScreen<'a> {
    handle: *mut xlib_sys::Screen,
    display: &'a XDisplay,
}

impl<'a> XScreen<'a> {
    /// Wraps a native X11 screen.
    ///
    /// # Arguments
    ///
    /// * `handle` - The native platform X11 pointer of the screen
    /// * `display` - The display the screen belongs to (and often represents entirely)
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
    pub unsafe fn new(handle: *mut xlib_sys::Screen, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Retrieves the underlying platform native X11 pointer.
    pub fn handle(&self) -> *mut xlib_sys::Screen {
        self.handle
    }

    /// Retrieves the number of the screen, usually 0.
    pub fn number(&self) -> i32 {
        unsafe { xlib_sys::XScreenNumberOfScreen(self.handle) }
    }

    /// Retrieves the root window of the screen.
    ///
    /// The root window is the top level background window which spans the entire screen.
    pub fn root_window(&self) -> XWindow {
        unsafe { XWindow::new((*self.handle).root, self.display) }
    }
}
