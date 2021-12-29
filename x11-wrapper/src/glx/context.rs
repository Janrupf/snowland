use crate::glx_sys;
use crate::XDisplay;
use crate::XDrawable;

#[derive(Debug)]
pub struct GLXContext<'a> {
    handle: glx_sys::GLXContext,
    display: &'a XDisplay,
}

impl<'a> GLXContext<'a> {
    pub unsafe fn new(handle: glx_sys::GLXContext, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    pub fn is_direct(&self) -> bool {
        (unsafe { glx_sys::glXIsDirect(self.display.handle(), self.handle) }) > 0
    }

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

    pub fn swap_buffers<D>(&self, drawable: &D)
    where
        D: XDrawable<'a>,
    {
        unsafe { glx_sys::glXSwapBuffers(self.display.handle(), drawable.drawable_handle()) }
    }

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
