use crate::{GLXError, XDisplay};

use crate::{XVisual, XVisualInfo};
use x11::glx as glx_sys;
use x11::glx::{glXGetFBConfigAttrib, glXGetVisualFromFBConfig, GLX_BAD_ATTRIBUTE};
use x11::xlib::XFree;

#[derive(Debug)]
pub struct GLXFBConfigArray<'a> {
    count: usize,
    handle: *mut glx_sys::GLXFBConfig,
    display: &'a XDisplay,
}

impl<'a> GLXFBConfigArray<'a> {
    pub unsafe fn wrap(
        count: usize,
        handle: *mut glx_sys::GLXFBConfig,
        display: &'a XDisplay,
    ) -> Self {
        Self {
            count,
            handle,
            display,
        }
    }

    pub fn handle(&self) -> *mut glx_sys::GLXFBConfig {
        self.handle
    }

    pub fn config_at(&self, index: usize) -> Option<GLXFBConfig> {
        if index >= self.count {
            None
        } else {
            let item = unsafe { GLXFBConfig::new(*self.handle.add(index), self.display) };
            Some(item)
        }
    }

    pub fn iter(&self) -> GLXFBConfigArrayIter {
        GLXFBConfigArrayIter::new(self)
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'a> Drop for GLXFBConfigArray<'a> {
    fn drop(&mut self) {
        unsafe { XFree(self.handle as _) };
        self.display.sync(false);
    }
}

pub struct GLXFBConfigArrayIter<'a> {
    array: &'a GLXFBConfigArray<'a>,
    pos: usize,
}

impl<'a> GLXFBConfigArrayIter<'a> {
    fn new(array: &'a GLXFBConfigArray<'a>) -> Self {
        Self { array, pos: 0 }
    }
}

impl<'a> Iterator for GLXFBConfigArrayIter<'a> {
    type Item = GLXFBConfig<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.array.config_at(self.pos).map(|config| {
            self.pos += 1;
            config
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.array.count - self.pos;

        (remaining, Some(remaining))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.array.count - self.pos
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.array.config_at(self.array.count - 1)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.array.config_at(n)
    }
}

#[derive(Debug, Clone)]
pub struct GLXFBConfig<'a> {
    handle: glx_sys::GLXFBConfig,
    display: &'a XDisplay,
}

impl<'a> GLXFBConfig<'a> {
    pub unsafe fn new(handle: glx_sys::GLXFBConfig, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    pub fn get_visual(&self) -> Option<XVisualInfo> {
        let handle = unsafe { glXGetVisualFromFBConfig(self.display.handle(), self.handle) };

        if handle.is_null() {
            None
        } else {
            Some(unsafe { XVisualInfo::new(handle, XVisual::new((*handle).visual)) })
        }
    }

    pub fn get_attrib(&self, attrib: i32) -> Result<i32, GLXError> {
        let mut value = 0;
        let ok =
            unsafe { glXGetFBConfigAttrib(self.display.handle(), self.handle, attrib, &mut value) };

        match ok {
            0 => Ok(value),
            GLX_BAD_ATTRIBUTE => Err(GLXError::BadAttribute(attrib)),
            v => Err(GLXError::GenericError(v)),
        }
    }

    pub fn handle(&self) -> glx_sys::GLXFBConfig {
        self.handle
    }

    pub fn extend_lifetime(self, display: &XDisplay) -> GLXFBConfig {
        assert_eq!(self.display.handle(), display.handle());

        GLXFBConfig {
            display,
            handle: self.handle,
        }
    }
}
