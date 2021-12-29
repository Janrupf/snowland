use std::fmt::{Debug, Formatter, Pointer};
use std::marker::PhantomData;
use x11::xlib as xlib_sys;

pub struct XVisual<'a> {
    handle: *mut xlib_sys::Visual,
    _data: PhantomData<&'a ()>,
}

impl<'a> XVisual<'a> {
    pub unsafe fn new(handle: *mut xlib_sys::Visual) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    pub fn id(&self) -> xlib_sys::VisualID {
        unsafe { xlib_sys::XVisualIDFromVisual(self.handle) }
    }
}

impl<'a> Debug for XVisual<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handle = if self.handle.is_null() {
            None
        } else {
            Some(unsafe { &*self.handle })
        };

        f.debug_struct("XVisual")
            .field("handle", &handle)
            .field("_data", &self._data)
            .finish()
    }
}

pub struct XVisualInfo<'a> {
    handle: *mut xlib_sys::XVisualInfo,
    visual: XVisual<'a>,
    _data: PhantomData<&'a ()>,
}

impl<'a> XVisualInfo<'a> {
    pub unsafe fn new(handle: *mut xlib_sys::XVisualInfo, visual: XVisual<'a>) -> Self {
        Self {
            handle,
            visual,
            _data: PhantomData,
        }
    }

    pub fn depth(&self) -> i32 {
        unsafe { (*(self.handle)).depth }
    }

    pub fn visual(&self) -> &XVisual {
        &self.visual
    }

    pub fn handle(&self) -> *mut xlib_sys::XVisualInfo {
        self.handle
    }
}

impl<'a> Drop for XVisualInfo<'a> {
    fn drop(&mut self) {
        unsafe { xlib_sys::XFree(self.handle as _) };
    }
}

impl<'a> Debug for XVisualInfo<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handle = if self.handle.is_null() {
            None
        } else {
            Some(unsafe { &*self.handle })
        };

        f.debug_struct("XVisualInfo")
            .field("handle", &handle)
            .field("_data", &self._data)
            .finish()
    }
}

impl<'a> PartialEq<Self> for XVisualInfo<'a> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.handle == *other.handle }
    }
}

impl<'a> Eq for XVisualInfo<'a> {}
