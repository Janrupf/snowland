use crate::glx_sys;
use crate::XDisplay;
use crate::XDrawable;

/// A GLX OpenGL context bound to a specific display.
///
/// Note that this struct does not guarantee that the context is the current context!
#[derive(Debug)]
pub struct GLXContext<'a> {
    handle: glx_sys::GLXContext,
    display: &'a XDisplay,
}

impl<'a> GLXContext<'a> {
    /// Creates a new GLX context.
    ///
    /// # Arguments
    ///
    /// * `handle` - The underlying native GLX context
    /// * `display` - The display the context was created on
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that `handle` is a valid GLX context.
    pub unsafe fn new(handle: glx_sys::GLXContext, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Determines whether this context uses direct rendering.
    pub fn is_direct(&self) -> bool {
        (unsafe { glx_sys::glXIsDirect(self.display.handle(), self.handle) }) > 0
    }

    /// Makes the context current using the given drawable.
    ///
    /// # Arguments
    ///
    /// * `drawable` - An X11 drawable such as a window or a pixmap
    pub fn make_current<D>(&self, drawable: &D)
    where
        D: XDrawable<'a>,
    {
        unsafe {
            glx_sys::glXMakeCurrent(
                self.display.handle(),
                drawable.drawable_handle(),
                self.handle,
            )
        };
    }

    /// Swaps the front/back buffers of the given drawable.
    ///
    /// The context has to be the current context and be active on the given drawable in order for
    /// the call to succeed!
    ///
    /// # Arguments
    ///
    /// * `drawable` - An X11 drawable such as a window or a pixmap
    pub fn swap_buffers<D>(&self, drawable: &D)
    where
        D: XDrawable<'a>,
    {
        unsafe { glx_sys::glXSwapBuffers(self.display.handle(), drawable.drawable_handle()) }
    }

    /// Removes the current context from the display.
    pub fn make_non_current(&self) {
        unsafe { glx_sys::glXMakeCurrent(self.display.handle(), 0, std::ptr::null_mut()) };
    }
}

impl<'a> Drop for GLXContext<'a> {
    fn drop(&mut self) {
        unsafe {
            glx_sys::glXMakeCurrent(self.display.handle(), 0, std::ptr::null_mut());
            glx_sys::glXDestroyContext(self.display.handle(), self.handle);
        }
    }
}
