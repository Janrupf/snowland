use crate::{GLXError, XDisplay};

use crate::glx_sys;
use crate::{XVisual, XVisualInfo};
use x11::glx::{glXGetFBConfigAttrib, glXGetVisualFromFBConfig, GLX_BAD_ATTRIBUTE};
use x11::xlib::XFree;

/// Wrapped array of GLX framebuffer configurations.
#[derive(Debug)]
pub struct GLXFBConfigArray<'a> {
    count: usize,
    handle: *mut glx_sys::GLXFBConfig,
    display: &'a XDisplay,
}

impl<'a> GLXFBConfigArray<'a> {
    /// Wraps the native platform representation of an array of GLX framebuffer configurations.
    ///
    /// # Arguments
    ///
    /// * `count` - The amount of configurations in the array
    /// * `handle` - Pointer to the first element of the array
    /// * `display` - The display the framebuffer configurations belong to
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that all parameters are correct.
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

    /// Retrieves the pointer to the first element of the array.
    pub fn handle(&self) -> *mut glx_sys::GLXFBConfig {
        self.handle
    }

    /// Attempts to retrieve a configuration of a specific index.
    ///
    /// # Arguments
    ///
    /// * `index` - The 0 based index of the configuration to retrieve
    pub fn config_at(&self, index: usize) -> Option<GLXFBConfig> {
        if index >= self.count {
            None
        } else {
            let item = unsafe { GLXFBConfig::new(*self.handle.add(index), self.display) };
            Some(item)
        }
    }

    /// Creates an iterator over all elements of the array.
    pub fn iter(&self) -> GLXFBConfigArrayIter {
        GLXFBConfigArrayIter::new(self)
    }

    /// Retrieves the amount of configurations stored in the array.
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

/// Iterator over a GLX framebuffer configuration array.
pub struct GLXFBConfigArrayIter<'a> {
    array: &'a GLXFBConfigArray<'a>,
    pos: usize,
}

impl<'a> GLXFBConfigArrayIter<'a> {
    /// Creates a new iterator over an existing GLX framebuffer configuration array.
    ///
    /// # Arguments
    ///
    /// * `array` - The array to iterate over
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

/// GLX framebuffer configuration.
#[derive(Debug, Clone)]
pub struct GLXFBConfig<'a> {
    handle: glx_sys::GLXFBConfig,
    display: &'a XDisplay,
}

impl<'a> GLXFBConfig<'a> {
    /// Wraps a native platform configuration pointer.
    ///
    /// # Arguments
    /// * `handle` - The underlying native platform configuration pointer
    /// * `display` - The display the configuration belongs to
    ///
    /// # Safety
    ///
    /// Its up to the caller to ensure that all parameters are valid.
    pub unsafe fn new(handle: glx_sys::GLXFBConfig, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Attempts to retrieve the underlying X11 visual information for the framebuffer configuration.
    pub fn get_visual(&self) -> Option<XVisualInfo> {
        let handle = unsafe { glXGetVisualFromFBConfig(self.display.handle(), self.handle) };

        if handle.is_null() {
            None
        } else {
            Some(unsafe { XVisualInfo::new(handle, XVisual::new((*handle).visual)) })
        }
    }

    /// Queries an attribute of the configuration.
    ///
    /// # Arguments
    ///
    /// * `attrib` - The attribute to query
    pub fn get_attribute(&self, attrib: i32) -> Result<i32, GLXError> {
        let mut value = 0;
        let ok =
            unsafe { glXGetFBConfigAttrib(self.display.handle(), self.handle, attrib, &mut value) };

        match ok {
            0 => Ok(value),
            GLX_BAD_ATTRIBUTE => Err(GLXError::BadAttribute(attrib)),
            v => Err(GLXError::GenericError(v)),
        }
    }

    /// Retrieves the underlying native platform pointer.
    pub fn handle(&self) -> glx_sys::GLXFBConfig {
        self.handle
    }

    /// Helper function to extend the lifetime of the configuration.
    ///
    /// When a configuration is retrieved from an array, it's lifetime is bound to that of the array.
    /// However, framebuffer configurations stay at least valid for the duration of the display
    /// connection.
    ///
    /// # Panics
    ///
    /// If an attempt is made to extend the lifetime using an unrelated display connection.
    pub fn extend_lifetime(self, display: &XDisplay) -> GLXFBConfig {
        assert_eq!(self.display.handle(), display.handle());

        GLXFBConfig {
            display,
            handle: self.handle,
        }
    }
}
